/// Provides an execute handler for pipelines.
pub trait Execute {
    /// Execute a pipeline call to this instance.
    /// Responsible for invoking the relevant handler(s).
    fn execute(self);
}

/// Provides an execute handler for pipelines, with a argument of type `TArg`.
pub trait ExecuteWith<TArg: ?Sized> {
    /// Execute a pipeline call to this instance with an argument.
    /// Responsible for invoking the relevant handler(s).
    fn execute_with(self, arg: &TArg);
}

/// Provides an Execute handler for pipelines, with a mutable argument of type `TArg`.
pub trait ExecuteWithMut<TArg: ?Sized> {
    /// Execute a pipeline call to this instance with a mutable argument.
    /// Responsible for invoking the relevant handler(s).
    fn execute_with_mut(self, arg: &mut TArg);
}

/// Blanket implementation of the [`Execute`] trait for any type
/// that can be converted to an [`Iterator`] over some type that
/// also implements [`Execute`]
///
/// ## Example
///
/// ```
/// use enum_pipeline::Execute;
/// use std::cell::RefCell;
///
/// enum Operations<'a> {
///     AddOne(&'a RefCell<u32>),
///     AddTwo(&'a RefCell<u32>),
/// }
///
/// impl Execute for Operations<'_> {
///     fn execute(self) {
///         match self {
///             Operations::AddOne(cell) => *cell.borrow_mut() += 1,
///             Operations::AddTwo(cell) => *cell.borrow_mut() += 2,
///         }
///     }
/// }
///
/// let acc = RefCell::new(0u32);
/// let my_op_pipeline = vec![
///     Operations::AddOne(&acc),
///     Operations::AddTwo(&acc),
///     Operations::AddTwo(&acc),
/// ];
///
/// my_op_pipeline.execute();
/// assert_eq!(5, *acc.borrow());
/// ```
impl<T> Execute for T
where
    T: IntoIterator,
    T::Item: Execute,
{
    fn execute(self) {
        // This is morally equivalent to a for loop or
        // a while let binding, but [`for_each`] has the opportunity
        // to be quicker in some cases if `T` is an adapter
        // like [`Chain`]
        self.into_iter().for_each(move |item| item.execute());
    }
}

/// Blanket implementation of the [`ExecuteWith`] trait for any type
/// that can be converted to an [`Iterator`] over some type that
/// also implements [`ExecuteWith`]
///
/// ## Example
///
/// ```
/// use enum_pipeline::ExecuteWith;
/// use std::cell::RefCell;
///
/// enum Operations {
///     Allocate(f32, f32),
///     Init,
///     Run(f32),
/// }
///
/// impl ExecuteWith<RefCell<String>> for Operations {
///     fn execute_with(self, arg: &RefCell<String>) {
///         match self {
///             Operations::Allocate(_, _) => arg.borrow_mut().push_str("[alloc]"),
///             Operations::Init => arg.borrow_mut().push_str("[init]"),
///             Operations::Run(_) => arg.borrow_mut().push_str("[run]"),
///         }
///     }
/// }
///
/// let my_op_pipeline = vec![
///     Operations::Init,
///     Operations::Allocate(1.0, 1.0),
///     Operations::Run(1.0),
/// ];
///
/// let arg = RefCell::new(String::from(""));
/// my_op_pipeline.execute_with(&arg);
/// assert_eq!(*arg.borrow(), String::from("[init][alloc][run]"));
/// ```
impl<T, TArg: ?Sized> ExecuteWith<TArg> for T
where
    T: IntoIterator,
    T::Item: ExecuteWith<TArg>,
{
    fn execute_with(self, arg: &TArg) {
        // This is morally equivalent to a for loop or
        // a while let binding, but [`for_each`] has the opportunity
        // to be quicker in some cases if `T` is an adapter
        // like [`Chain`]
        self.into_iter()
            .for_each(move |item| item.execute_with(arg));
    }
}

/// Blanket implementation of the [`ExecuteWithMut`] trait for any type
/// that can be converted to an [`Iterator`] over some type that
/// also implements [`ExecuteWithMut`]
///
/// ## Example
///
/// ```
/// use enum_pipeline::ExecuteWithMut;
///
/// enum Operations {
///     Allocate(f32, f32),
///     Init,
///     Run(f32),
/// }
///
/// impl<T> ExecuteWithMut<T> for Operations where T: std::ops::AddAssign<u32> {
///     fn execute_with_mut(self, arg: &mut T) {
///         match self {
///             Operations::Allocate(_, _) => *arg += 2,
///             Operations::Init => *arg += 3,
///             Operations::Run(_) => *arg += 5,
///         }
///     }
/// }
///
/// fn do_work_with_mut() {
///     let my_op_pipeline = vec![
///         Operations::Init,
///         Operations::Allocate(1.0, 1.0),
///         Operations::Run(1.0),
///     ];
///
///     let mut acc = 0;
///     my_op_pipeline.execute_with_mut(&mut acc);
///     assert_eq!(acc, 10);
/// }
/// ```
impl<T, TArg: ?Sized> ExecuteWithMut<TArg> for T
where
    T: IntoIterator,
    T::Item: ExecuteWithMut<TArg>,
{
    fn execute_with_mut(self, arg: &mut TArg) {
        // This is morally equivalent to a for loop or
        // a while let binding, but [`for_each`] has the opportunity
        // to be quicker in some cases if `T` is an adapter
        // like [`Chain`]
        self.into_iter()
            .for_each(move |item| item.execute_with_mut(arg));
    }
}

#[cfg(test)]
mod tests {
    use crate::{Execute, ExecuteWith, ExecuteWithMut};
    use enum_pipeline_derive::{Execute, ExecuteWith, ExecuteWithMut};

    #[derive(Execute)]
    enum VoidDispatchPipeline {
        #[handler(VoidDispatchPipeline::handle_one)]
        One,
        #[handler(handle_two)]
        Two,
    }

    static mut VOID_ONE_COUNT: i32 = 0;
    static mut VOID_TWO_COUNT: i32 = 0;

    impl VoidDispatchPipeline {
        fn handle_one() {
            unsafe {
                VOID_ONE_COUNT += 1;
            }
        }

        fn handle_two() {
            unsafe {
                VOID_TWO_COUNT += 1;
            }
        }
    }

    #[test]
    fn void_dispatch_works() {
        let pipeline = vec![VoidDispatchPipeline::One, VoidDispatchPipeline::Two];

        pipeline.execute();

        unsafe {
            assert_eq!(1, VOID_ONE_COUNT);
            assert_eq!(1, VOID_TWO_COUNT);
        }
    }

    enum RefDataPipeline {
        One(i32),
        Two,
    }

    static mut REF_ONE_VALUE: i32 = 0;
    static mut REF_TWO_COUNT: i32 = 0;

    struct RefDataPipelineData {
        mult: i32,
    }

    impl RefDataPipeline {
        fn handle_one(v: i32, arg: &RefDataPipelineData) {
            unsafe {
                REF_ONE_VALUE += v * arg.mult;
            }
        }

        fn handle_two(_arg: &RefDataPipelineData) {
            unsafe {
                REF_TWO_COUNT += 1;
            }
        }
    }

    impl ExecuteWith<RefDataPipelineData> for RefDataPipeline {
        fn execute_with(self, arg: &RefDataPipelineData) {
            match self {
                RefDataPipeline::One(f) => RefDataPipeline::handle_one(f, arg),
                RefDataPipeline::Two => RefDataPipeline::handle_two(arg),
            }
        }
    }

    #[test]
    fn ref_data_pipeline_works() {
        let pipeline = vec![RefDataPipeline::One(24), RefDataPipeline::Two];

        let data = RefDataPipelineData { mult: 2 };

        pipeline.execute_with(&data);

        unsafe {
            assert_eq!(48, REF_ONE_VALUE);
            assert_eq!(1, REF_TWO_COUNT);
        }
    }

    enum MutDataPipeline {
        One(i32),
        Two,
    }

    #[derive(Default)]
    struct MutDataPipelineData {
        one_value: i32,
        two_count: i32,
    }

    // no macro yet, srry
    impl ExecuteWithMut<MutDataPipelineData> for MutDataPipeline {
        fn execute_with_mut(self, arg: &mut MutDataPipelineData) {
            match self {
                MutDataPipeline::One(i) => arg.one_value += i,
                MutDataPipeline::Two => arg.two_count += 1,
            }
        }
    }

    #[test]
    fn mut_data_pipeline_works() {
        let pipeline = vec![MutDataPipeline::One(12), MutDataPipeline::Two];

        let mut data = MutDataPipelineData::default();
        pipeline.execute_with_mut(&mut data);

        assert_eq!(12, data.one_value);
        assert_eq!(1, data.two_count);
    }

    struct MacroRefPipelineData {}

    #[derive(ExecuteWith)]
    #[execute_with(MacroRefPipelineData)]
    enum MacroRefPipeline {
        #[handler(handle_a)]
        A,
        #[handler(handle_b)]
        B,
    }

    static mut MACRO_REF_ONE_COUNT: i32 = 0;
    static mut MACRO_REF_TWO_COUNT: i32 = 0;

    impl MacroRefPipeline {
        fn handle_a(_data: &MacroRefPipelineData) {
            unsafe {
                MACRO_REF_ONE_COUNT += 1;
            }
        }

        fn handle_b(_data: &MacroRefPipelineData) {
            unsafe {
                MACRO_REF_TWO_COUNT += 1;
            }
        }
    }

    #[test]
    fn macro_ref_pipeline_works() {
        vec![MacroRefPipeline::A, MacroRefPipeline::B].execute_with(&MacroRefPipelineData {});

        unsafe {
            assert_eq!(1, MACRO_REF_ONE_COUNT);
            assert_eq!(1, MACRO_REF_TWO_COUNT);
        }
    }

    #[derive(Default)]
    struct MacroMutRefData {
        a_count: i32,
        b_count: i32,
    }

    #[derive(ExecuteWithMut)]
    #[execute_with(MacroMutRefData)]
    enum MacroMutRefPipeline {
        #[handler(handle_a)]
        A(i32),
        #[handler(handle_b)]
        B,
    }

    impl MacroMutRefPipeline {
        fn handle_a(_i: i32, arg: &mut MacroMutRefData) {
            arg.a_count += 1;
        }

        fn handle_b(arg: &mut MacroMutRefData) {
            arg.b_count += 1;
        }
    }

    #[test]
    fn macro_mut_pipeline_works() {
        let mut arg = MacroMutRefData::default();
        vec![MacroMutRefPipeline::A(23), MacroMutRefPipeline::B].execute_with_mut(&mut arg);

        assert_eq!(1, arg.a_count);
        assert_eq!(1, arg.b_count);
    }

    #[derive(Default)]
    struct MutRefData {
        a_count: i32,
        b_count: i32,
    }

    enum MutRefPipeline {
        A(i32),
        B,
    }

    impl ExecuteWithMut<MutRefData> for MutRefPipeline {
        fn execute_with_mut(self, arg: &mut MutRefData) {
            match self {
                MutRefPipeline::A(_i) => arg.a_count += 1,
                MutRefPipeline::B => arg.b_count += 1,
            }
        }
    }

    #[test]
    fn _mut_pipeline_works() {
        let mut arg = MutRefData::default();
        vec![MutRefPipeline::A(23), MutRefPipeline::B].execute_with_mut(&mut arg);

        assert_eq!(1, arg.a_count);
        assert_eq!(1, arg.b_count);
    }
}

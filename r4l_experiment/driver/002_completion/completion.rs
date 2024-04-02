#![no_std]
#![feature(allocator_api, global_asm)]

use kernel::prelude::*;
use kernel::chrdev::{ChrDev, Cdev, CdevRegistration, File, FileOpener, FileOperations};
use kernel::sync::completion;

module! {
    type: MyModule,
    name: b"completion_example",
    author: b"tester",
    description: b"Example of Kernel's completion mechanism in Rust",
    license: b"GPL",
}

struct MyDevice {
    completion: completion::Completion,
}

impl FileOperations for MyDevice {
    kernel::declare_file_operations!(read, write);
}

impl FileOpener<MyDevice> for MyDevice {
    fn open(ctx: &kernel::file_operations::FileOpenContext) -> Result<Self::Wrapper> {
        pr_info!("open() is invoked\n");
        Ok(MyDevice { completion: completion::Completion::new() }.into())
    }
}

impl MyDevice {
    fn read(&self, _file: &File, _data: &mut kernel::user_ptr::UserSlicePtrWriter, _offset: &mut u64) -> Result<usize> {
        pr_info!("read() is invoked\n");
        self.completion.wait();
        pr_info!("awoken\n");
        Ok(0)
    }

    fn write(&self, _file: &File, _data: &kernel::user_ptr::UserSlicePtrReader, _offset: &mut u64) -> Result<usize> {
        pr_info!("write() is invoked\n");
        self.completion.complete();
        Ok(0)
    }
}

struct MyModule {
    _chrdev: CdevRegistration<MyDevice>,
}

impl KernelModule for MyModule {
    fn init() -> Result<Self> {
        pr_info!("completion_example module loaded\n");
        let chrdev = CdevRegistration::new_pinned::<MyDevice>(cstr!("completion_example"), None)?;
        Ok(MyModule { _chrdev: chrdev })
    }
}

impl Drop for MyModule {
    fn drop(&mut self) {
        pr_info!("completion_example module unloaded\n");
    }
}

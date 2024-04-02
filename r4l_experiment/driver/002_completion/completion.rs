#![no_std]
#![feature(allocator_api, global_asm)]

use kernel::prelude::*;
use kernel::sync::completion;
use kernel::file::File;
use kernel::file_operations::{FileOpener, FileOperations};
use kernel::cdev::{Cdev, CdevRegistration};

module! {
    type: CompletionModule,
    name: b"completion",
    author: b"Tester",
    description: b"Example of Kernel's completion mechanism",
    license: b"GPL",
}

struct CompletionDevice {
    cdev: Cdev,
    completion: completion::Completion,
}

impl FileOperations for CompletionDevice {
    kernel::declare_file_operations!(read, write);
}

impl FileOpener<CompletionDevice> for CompletionDevice {
    fn open(ctx: &kernel::file_operations::FileOpenContext) -> Result<Self::Wrapper> {
        pr_info!("{}() is invoked\n", ctx.name());
        Ok(CompletionDevice {
            cdev: Cdev::new(),
            completion: completion::Completion::new(),
        }
        .into())
    }
}

impl CompletionDevice {
    fn read(&self, _file: &File, _buf: &mut [u8], _offset: u64) -> Result<usize> {
        pr_info!("{}() is invoked\n", "completion_read");

        pr_info!("process {}({}) is going to sleep\n", current_pid(), current_comm());
        self.completion.wait();
        pr_info!("awoken {}({})\n", current_pid(), current_comm());

        Ok(0)
    }

    fn write(&self, _file: &File, _buf: &[u8], _offset: u64) -> Result<usize> {
        pr_info!("{}() is invoked\n", "completion_write");

        pr_info!("process {}({}) awakening the readers...\n", current_pid(), current_comm());
        self.completion.complete();

        Ok(_buf.len())
    }
}

struct CompletionModule {
    chrdev: CdevRegistration<CompletionDevice>,
}

impl KernelModule for CompletionModule {
    fn init() -> Result<Self> {
        pr_info!("{} module loaded\n", module_name!());

        let chrdev = CdevRegistration::new_pinned::<CompletionDevice>(cstr!("completion"), None)?;
        Ok(CompletionModule { chrdev })
    }
}

impl Drop for CompletionModule {
    fn drop(&mut self) {
        pr_info!("{} module unloaded\n", module_name!());
    }
}

fn current_pid() -> i32 {
    kernel::task::current().pid().into()
}

fn current_comm() -> &'static str {
    kernel::task::current().comm()
}

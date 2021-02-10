use nix::unistd::{pipe, fork, ForkResult, close, dup2, execvp};
use std::ffi::CString;
use nix::sys::wait::waitpid;

fn main() {
    println!("Executes ls -l | grep Cargo | wc");

    let ls = vec![CString::new("ls").unwrap(), CString::new("-l").unwrap()];
    let grep = vec![CString::new("grep").unwrap(), CString::new("Cargo").unwrap()];
    let wc = vec![CString::new("wc").unwrap()];

    match unsafe {fork()}.unwrap() {
        ForkResult::Parent{child} => {
            println!("wc pid is {}", child);
            waitpid(child, Option::None).unwrap();
            println!("Finished! Exiting...");
        },

        ForkResult::Child => {
            let (wc_in, grep_out) = pipe().unwrap();

            match unsafe {fork()}.unwrap() {
                ForkResult::Parent{child} => {
                    close(grep_out).unwrap();

                    println!("grep pid is {}", child);
                    dup2(wc_in, 0).unwrap();
                    let array = wc.into_boxed_slice();
                    execvp(&array[0], &*array).unwrap();
                },

                ForkResult::Child => {
                    close(wc_in).unwrap();

                    let (grep_in, ls_out) = pipe().unwrap();

                    match unsafe {fork()}.unwrap() {
                        ForkResult::Parent {child} => {
                            close(ls_out).unwrap();

                            println!("ls pid is {}", child);
                            dup2(grep_out, 1).unwrap();
                            dup2(grep_in, 0).unwrap();
                            let array = grep.into_boxed_slice();
                            execvp(&array[0], &*array).unwrap();
                        },

                        ForkResult::Child => {
                            close(grep_in).unwrap();

                            dup2(ls_out, 1).unwrap();
                            let array = ls.into_boxed_slice();
                            execvp(&array[0], &*array).unwrap();
                        }
                    }
                }
            }
        }
    }
}

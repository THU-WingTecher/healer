#[cfg(not(target_os = "windows"))]
use std::os::unix::io::{FromRawFd, IntoRawFd};
#[cfg(target_os = "windows")]
use std::os::windows::io::{FromRawHandle, IntoRawHandle};
use std::thread;
use std::{fs::File, future::pending};
use std::{
    io::ErrorKind,
    sync::{
        mpsc::{channel, Receiver},
        Once,
    },
};
use tokio::runtime::{Builder, Runtime};

static mut RUNTIME: Option<Runtime> = None;
static ONCE: Once = Once::new();

pub fn init_runtime() {
    ONCE.call_once(|| {
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to init tokio runtime.");
        unsafe {
            RUNTIME = Some(rt);
        }
        thread::Builder::new()
            .name("healer-bg-tasks".into())
            .spawn(move || {
                runtime().block_on(pending::<()>());
            })
            .expect("Failed to spawn healer background thread");
    })
}

pub fn runtime() -> &'static Runtime {
    unsafe { RUNTIME.as_ref().unwrap() }
}

pub struct Reader {
    recv: Receiver<Vec<u8>>,
}

impl Reader {
    #[cfg(target_os = "windows")]
    pub fn new<F: IntoRawHandle>(f: F) -> Self {
        let f = f.into_raw_handle();
        let recv = Self::read_to_end_inner(unsafe { File::from_raw_handle(f) });
        Self { recv }
    }

    #[cfg(target_os = "unix")]
    pub fn new<F: IntoRawFd>(f: F) -> Self {
        let f = f.into_raw_fd();
        let recv = Self::read_to_end_inner(unsafe { File::from_raw_fd(f) });
        Self { recv }
    }

    pub fn recv_data(self) -> Vec<u8> {
        self.recv.recv().unwrap()
    }

    fn read_to_end_inner(f: File) -> Receiver<Vec<u8>> {
        use tokio::io::AsyncReadExt;

        let mut f = tokio::fs::File::from_std(f);
        let (sender, recv) = channel::<Vec<u8>>();

        runtime().spawn(async move {
            let mut buf: [(Vec<u8>, usize); 2] = [(vec![0; 2048], 0), (vec![0; 2048], 0)];
            loop {
                buf[0].1 = 0;
                let current_buf = &mut buf[0].0[..];
                let mut eof = false;
                let mut len = 0;
                while len != 2048 {
                    match f.read(&mut current_buf[len..]).await {
                        Ok(0) => {
                            eof = true;
                            break;
                        }
                        Ok(n) => len += n,
                        Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                        Err(_) => {
                            eof = true;
                            break;
                        }
                    }
                }
                buf[0].1 = len;
                if eof {
                    break;
                }
                buf.reverse();
            }
            let ret = buf[1].0[0..buf[1].1]
                .iter()
                .chain(&buf[0].0[0..buf[0].1])
                .copied()
                .collect::<Vec<_>>();

            sender.send(ret).unwrap();
        });
        recv
    }
}

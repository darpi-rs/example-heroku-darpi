use darpi::job::{CpuJob, FutureJob, IOBlockingJob, JobExt};
use darpi::{job_factory, Body, Response};

//FutureJob types are queued on the regular tokio runtime
// they are executed in the background and do not hold up the
// response to the user
#[job_factory(Request)]
async fn first_async_job() -> FutureJob {
    async { println!("first job in the background.") }.into()
}

// IOBlockingJob types are for any io operation that cannot be performed
// in an async context. They are offloaded to a thread that is ok to block.
#[job_factory(Response)]
async fn first_sync_job(#[response] r: &Response<Body>) -> IOBlockingJob {
    let status_code = r.status();
    {
        move || {
            std::thread::sleep(std::time::Duration::from_secs(2));
            println!(
                "first_sync_job in the background for a request with status {}",
                status_code
            );
        }
    }
    .into()
}

// CpuJob type is used for cpu bound tasks.
// they are being ran on the rayon runtime
#[job_factory(Response)]
async fn first_sync_job1() -> CpuJob {
    {
        || {
            let mut r = 0;
            for _ in 0..10000000 {
                r += 1;
            }
            println!("first_sync_job1 finished in the background. {}", r)
        }
    }
    .into()
}

#[job_factory(Response)]
async fn first_sync_io_job() -> IOBlockingJob {
    {
        || {
            std::thread::sleep(std::time::Duration::from_secs(2));
            println!("sync io finished in the background");
        }
    }
    .into()
}

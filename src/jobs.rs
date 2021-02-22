use darpi::job::{CpuJob, FutureJob, IOBlockingJob, JobExt};
use darpi::{job::Job, job_factory, Body, Response};

#[job_factory(Request)]
async fn first_async_job() -> FutureJob {
    async { println!("first job in the background.") }.into()
}

#[job_factory(Response)]
async fn first_sync_job(#[response] r: &Response<Body>) -> Job {
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
    .io_blocking()
}

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

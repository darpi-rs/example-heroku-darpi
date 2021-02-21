use darpi::futures::FutureExt;
use darpi::{job::Job, job_factory, Body, Response};

#[job_factory(Request)]
async fn first_async_job() -> Job {
    Job::Future(async { println!("first job in the background.") }.boxed())
}

#[job_factory(Response)]
async fn first_sync_job(#[response] r: &Response<Body>) -> Job {
    let status_code = r.status();
    Job::IOBlocking(Box::new(move || {
        std::thread::sleep(std::time::Duration::from_secs(2));
        println!(
            "first_sync_job in the background for a request with status {}",
            status_code
        );
    }))
}

#[job_factory(Response)]
async fn first_sync_job1() -> Job {
    Job::CpuBound(Box::new(|| {
        let mut r = 0;
        for _ in 0..10000000 {
            r += 1;
        }
        println!("first_sync_job1 finished in the background. {}", r)
    }))
}

#[job_factory(Response)]
async fn first_sync_io_job() -> Job {
    Job::IOBlocking(Box::new(|| {
        std::thread::sleep(std::time::Duration::from_secs(2));
        println!("sync io finished in the background");
    }))
}

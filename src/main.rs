use util::Result;

mod smoketest;
mod util;

#[tokio::main]
async fn main() -> Result<()> {
    smoketest::start("3000").await?;

    Ok(())
}

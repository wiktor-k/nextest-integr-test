#[cfg(test)]
mod tests {
    use rstest::rstest;
    use rustainers::{
        Container, ExposedPort, ImageName, RunnableContainer, RunnableContainerBuilder,
        ToRunnableContainer, WaitStrategy,
        runner::{RunOption, Runner},
    };
    use testresult::TestResult;
    use tracing_test::traced_test;
    use url::Url;
    use uuid::{NoContext, Timestamp, Uuid};

    const IMAGE_NAME: &str = "docker.io/nitrokey/nethsm:c16fe4ed";
    const DEFAULT_PORT: u16 = 8443;
    const DEFAULT_PATH: &str = "/api/v1";

    #[derive(Debug)]
    pub struct NetHsmImage {
        pub image: ImageName,
        pub port: ExposedPort,
    }

    impl NetHsmImage {
        pub async fn url(&self) -> TestResult<url::Url> {
            Ok(Url::parse(&format!(
                "https://localhost:{}{}",
                self.port.host_port().await?,
                DEFAULT_PATH
            ))?)
        }
    }

    impl Default for NetHsmImage {
        fn default() -> Self {
            Self {
                image: ImageName::new(IMAGE_NAME),
                port: ExposedPort::new(DEFAULT_PORT),
            }
        }
    }

    impl ToRunnableContainer for NetHsmImage {
        fn to_runnable(&self, builder: RunnableContainerBuilder) -> RunnableContainer {
            builder
                .with_image(self.image.clone())
                .with_container_name(Some(format!(
                    "nethsm-test-{}",
                    Uuid::new_v7(Timestamp::now(NoContext))
                )))
                // XXX: Using this wait strategy will make some tests stuck indefinitely
                //.with_wait_strategy(WaitStrategy::stderr_contains(
                //    "listening on 8443/TCP for HTTPS",
                //))
                .with_wait_strategy(WaitStrategy::HttpSuccess {
                    https: true,
                    path: "/".into(),
                    container_port: 8443.into(),
                })
                .with_port_mappings([self.port.clone()])
                .build()
        }
    }

    /// Creates and starts a new NetHSM container.
    pub async fn create_container(id: u32) -> TestResult<Container<NetHsmImage>> {
        let runner = Runner::podman()?;
        let image = NetHsmImage::default();
        println!("----------------> {id} image: {:#?}", image.image);
        let options = RunOption::builder()
            .with_name(format!(
                "www-test-{id}-{}",
                Uuid::new_v7(Timestamp::now(NoContext))
            ))
            .build();
        let container = runner.start_with_options(image, options).await?;
        println!("================> {id} trying to get the URL...");
        println!(
            "================> {id} serving URL: {}",
            container.url().await?
        );
        Ok(container)
    }

    #[traced_test]
    #[rstest]
    #[case(0)]
    #[case(1)]
    #[case(2)]
    #[case(3)]
    #[case(4)]
    #[case(5)]
    #[case(6)]
    #[case(7)]
    #[case(8)]
    #[case(9)]
    #[case(10)]
    #[case(11)]
    #[case(12)]
    #[case(13)]
    #[case(14)]
    #[case(15)]
    #[case(16)]
    #[case(17)]
    #[case(18)]
    #[case(19)]
    #[case(20)]
    #[tokio::test]
    async fn test_init(#[case] id: u32) -> TestResult {
        let _ = env_logger::builder().is_test(true).try_init();

        use std::time::Duration;

        let container = create_container(id).await?;
        let url = container.url().await?;
        println!("URL: {} for {}", url, id);
        tokio::time::sleep(Duration::from_secs(2)).await;

        let client = reqwest::ClientBuilder::new()
            .danger_accept_invalid_certs(true)
            .build()?;
        let resp = client
            .get(format!("{}/health/state", url))
            .send()
            .await?
            .text()
            .await?;
        println!("{id} RESP: {resp}");

        tokio::time::sleep(Duration::from_secs(10)).await;
        assert!(true);
        Ok(())
    }
}

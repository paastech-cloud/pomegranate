use bollard::container::ListContainersOptions;
use std::collections::HashMap;
use std::default::Default;

extern crate pomegranate;

async fn list_containers() {
    let mut filters = HashMap::new();
    filters.insert("health", vec!["unhealthy"]);

    let options = Some(ListContainersOptions {
        all: true,
        filters,
        ..Default::default()
    });

    let docker_engine = pomegranate::DockerEngine::new();
    println!("{:#?}", docker_engine.docker.list_containers(options).await);
}

#[test]
fn test_add() {
    let app = pomegranate::application::Application {
        application_id: "application".to_string(),
        project_id: "project".to_string(),
        image_name: "nginx".to_string(),
        image_tag: String::from("latest"),
        ..Default::default()
    };

    

    let docker_engine = pomegranate::DockerEngine::new();
    pomegranate::engine::Engine::start_application(&docker_engine, &app);
    list_containers();
    //println!(" Test : {:?}", docker_engine.docker.inspect_container());
    assert_eq!(list_containers(), "");
}

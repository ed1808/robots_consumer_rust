use robots_consumer::RobotConsumer;

fn main() {
    let mut consumer = RobotConsumer::new(
        String::from("robots_dirname"),
        String::from("robots_file.json"),
    );

    consumer.start();
}

use clap::Parser;

#[derive(Parser)]
#[command(author, version, about)]
#[command(arg_required_else_help(true))]
pub struct Cli {
    #[arg(
        long,
        env = "KAFKA_BOOTSTRAP_SERVERS",
        default_value = "kafka:9094",
        help = "Kafka Bootstrap Server"
    )]
    pub bootstrap_servers: String,
    #[arg(
        long,
        env = "KAFKA_TOPIC",
        default_value = "etl-processor_output",
        help = "Kafka Topic"
    )]
    pub topic: String,
    #[arg(
        long,
        env = "KAFKA_RESPONSE_TOPIC",
        default_value = "etl-processor_output_response",
        help = "Kafka Response Topic"
    )]
    pub response_topic: String,
    #[arg(
        long,
        env = "KAFKA_GROUP_ID",
        default_value = "mv64e-kafka-to-rest-gateway",
        help = "Kafka Group ID"
    )]
    pub group_id: String,
    #[arg(
        long,
        env = "DNPM_DIP_URI",
        help = "DNPM:DIP URI for API requests"
    )]
    pub dnpm_dip_uri: String,
    #[arg(
        long,
        env = "DNPM_DIP_USERNAME",
        help = "DNPM:DIP Username"
    )]
    pub dnpm_dip_username: Option<String>,
    #[arg(
        long,
        env = "DNPM_DIP_PASSWORD",
        help = "DNPM:DIP Password"
    )]
    pub dnpm_dip_password: Option<String>,
    #[arg(
        long,
        env = "KAFKA_SSL_CA_FILE",
        help = "CA file for SSL connection to Kafka"
    )]
    pub ssl_ca_file: Option<String>,
    #[arg(
        long,
        env = "KAFKA_SSL_CERT_FILE",
        help = "Certificate file for SSL connection to Kafka"
    )]
    pub ssl_cert_file: Option<String>,
    #[arg(
        long,
        env = "KAFKA_SSL_KEY_FILE",
        help = "Key file for SSL connection to Kafka"
    )]
    pub ssl_key_file: Option<String>,
    #[arg(long, env = "KAFKA_SSL_KEY_PASSWORD", help = "The SSL key password")]
    pub ssl_key_password: Option<String>,
}
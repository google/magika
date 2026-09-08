service "catalog" {
  image = "registry.example.test/catalog:1.4"

  resources {
    cpu    = 2
    memory = "512MiB"
  }

  environment = {
    LOG_LEVEL = "info"
    REGION    = "us-central1"
  }
}

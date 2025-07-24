# Deployment Configuration

This document outlines the deployment configurations managed by Repack for the test schema environments.

## Configuration Overview

The test schema demonstrates Repack's configuration management system with environment-specific deployment settings. These configurations are defined in [`test.repack`](test.repack) and can be used to generate environment-specific configuration files.

## Configuration Template

### ApiDeployment

**Purpose**: Deployment configuration template for API services

**Parameters**:
- **`host_ip`**: Server IP address for the API service
- **`db_username`**: Database connection username
- **`db_password`**: Database connection password

**Usage**: This template defines the structure for environment-specific configurations, ensuring consistency across all deployment environments.

## Environment Instances

### Production Environment (`@prod`)

**Instance Name**: `Production`  
**Environment Tag**: `@prod`

**Configuration Values**:
- **Host IP**: `192.168.0.1`
- **Database Username**: `admin`
- **Database Password**: `test`

**Purpose**: Production deployment configuration for live services

**Security Considerations**:
- This is a test configuration; production passwords should be properly secured
- Consider using environment variables or secret management systems
- Implement proper access controls and network security

### Staging Environment (`@staging`)

**Instance Name**: `Staging`  
**Environment Tag**: `@staging`

**Configuration Values**:
- **Host IP**: `10.0.0.1`
- **Database Username**: `admin2`
- **Database Password**: `test2`

**Purpose**: Staging environment for testing and validation before production deployment

**Usage**:
- Pre-production testing
- Integration testing
- User acceptance testing
- Performance testing with production-like data

## Configuration Generation

### Command Usage

Generate configuration files for specific environments:

```bash
# Generate production configuration
repack configure prod test.repack

# Generate staging configuration
repack configure staging test.repack

# Generate all configurations
repack configure test.repack
```

### Generated Output

When using configuration blueprints, these instances can generate:

- **Environment Files**: `.env` files with environment variables
- **Docker Compose**: Service configuration for containerized deployments
- **Kubernetes Manifests**: ConfigMaps and Secrets for Kubernetes deployments
- **Terraform Variables**: Infrastructure as Code configuration
- **Application Config**: JSON/YAML configuration files

### Example Generated Configuration

**Production Environment File (`prod.env`)**:
```bash
HOST_IP=192.168.0.1
DB_USERNAME=admin
DB_PASSWORD=test
ENVIRONMENT=production
```

**Staging Environment File (`staging.env`)**:
```bash
HOST_IP=10.0.0.1
DB_USERNAME=admin2
DB_PASSWORD=test2
ENVIRONMENT=staging
```

## Blueprint Integration

### Configuration Blueprints

Custom blueprints can iterate over configuration instances:

```blueprint
[meta id]env_config[/meta]
[meta name]Environment Configuration[/meta]
[meta kind]configure[/meta]

[each ApiDeployment]
[file][name.lowercase].env[/file]
# [name] Environment Configuration
HOST_IP=[host_ip]
DB_USERNAME=[db_username]
DB_PASSWORD=[db_password]
ENVIRONMENT=[environment]
[/each]
```

### Docker Compose Integration

```blueprint
[meta id]docker_compose[/meta]
[meta name]Docker Compose[/meta]
[meta kind]configure[/meta]

[each ApiDeployment]
[file]docker-compose.[name.lowercase].yml[/file]
version: '3.8'
services:
  api:
    image: myapp:latest
    environment:
      - HOST_IP=[host_ip]
      - DB_USERNAME=[db_username]
      - DB_PASSWORD=[db_password]
    ports:
      - "8080:8080"
  
  database:
    image: postgres:15
    environment:
      - POSTGRES_USER=[db_username]
      - POSTGRES_PASSWORD=[db_password]
      - POSTGRES_DB=myapp
[/each]
```

### Kubernetes ConfigMap

```blueprint
[meta id]k8s_config[/meta]
[meta name]Kubernetes Configuration[/meta]
[meta kind]configure[/meta]

[each ApiDeployment]
[file]k8s/[name.lowercase]-config.yaml[/file]
apiVersion: v1
kind: ConfigMap
metadata:
  name: [name.lowercase]-config
  namespace: default
data:
  HOST_IP: "[host_ip]"
  DB_USERNAME: "[db_username]"
---
apiVersion: v1
kind: Secret
metadata:
  name: [name.lowercase]-secret
  namespace: default
type: Opaque
stringData:
  DB_PASSWORD: "[db_password]"
[/each]
```

## Security Best Practices

### Password Management

**❌ Don't** (as shown in this test):
```repack
instance Production: ApiDeployment @prod {
    db_password "plaintext_password"  // Insecure
}
```

**✅ Do** (recommended for production):
```repack
instance Production: ApiDeployment @prod {
    db_password "${DB_PASSWORD_PROD}"  // Environment variable
}
```

### Environment Variable Integration

Use environment variables in configuration values:

```repack
instance Production: ApiDeployment @prod {
    host_ip "${PROD_HOST_IP}"
    db_username "${PROD_DB_USER}"
    db_password "${PROD_DB_PASSWORD}"
}
```

### Secret Management

For production systems:

1. **Use Secret Management Systems**: AWS Secrets Manager, HashiCorp Vault, Azure Key Vault
2. **Environment Variables**: Never commit secrets to version control
3. **Config Maps vs Secrets**: Use Kubernetes Secrets for sensitive data
4. **Encryption**: Encrypt configuration files at rest
5. **Access Controls**: Implement proper RBAC for configuration access

## Deployment Workflow

### Development to Production

1. **Development**: Use local configuration with development database
2. **Staging**: Deploy to staging environment with staging configuration
3. **Testing**: Validate application behavior in staging
4. **Production**: Deploy to production with production configuration

### CI/CD Integration

```yaml
# Example GitHub Actions workflow
name: Deploy
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Generate Configuration
        run: |
          repack configure staging schema.repack
          repack configure prod schema.repack
      
      - name: Deploy to Staging
        run: |
          # Deploy using generated staging configuration
          docker-compose -f docker-compose.staging.yml up -d
      
      - name: Deploy to Production
        if: github.ref == 'refs/heads/main'
        run: |
          # Deploy using generated production configuration
          kubectl apply -f k8s/production-config.yaml
```

## Monitoring and Observability

### Configuration Tracking

Track configuration changes:

1. **Version Control**: Keep configuration schemas in git
2. **Change Logs**: Document configuration changes
3. **Deployment History**: Track which configurations were deployed when
4. **Rollback Capability**: Maintain ability to rollback configuration changes

### Health Checks

Implement health checks that validate configuration:

```yaml
# Example health check configuration
health_check:
  database:
    host: ${HOST_IP}
    username: ${DB_USERNAME}
    password: ${DB_PASSWORD}
    timeout: 5s
  
  api:
    endpoint: "http://${HOST_IP}:8080/health"
    timeout: 3s
```

## Migration and Updates

### Configuration Schema Evolution

When updating configuration schemas:

1. **Backward Compatibility**: Ensure existing configurations remain valid
2. **Migration Scripts**: Provide scripts to update existing configurations
3. **Validation**: Implement configuration validation
4. **Testing**: Test configuration changes in staging first

### Environment Parity

Maintain consistency across environments:

- **Structure**: Same configuration structure across all environments
- **Validation**: Same validation rules for all environments
- **Monitoring**: Consistent monitoring across environments
- **Documentation**: Keep environment-specific documentation updated

This deployment configuration system demonstrates how Repack can manage complex, multi-environment deployments while maintaining security, consistency, and reliability.
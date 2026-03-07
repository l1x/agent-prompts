```yaml
kind: prompt
name: aws
description: "AWS conventions with Terraform for IaC and AWS CLI for operational tasks"
inputs: []
outputs: []
```

## AWS Conventions

- Use Terraform for all infrastructure — no manual console changes
- Use AWS CLI only for operations Terraform cannot handle (debugging, log tailing, one-off queries, incident response)
- Follow the principle of least privilege for all IAM roles and policies
- Use resource tagging consistently: `Environment`, `Service`, `Owner`, `ManagedBy`
- Prefer managed services over self-hosted (RDS over self-managed Postgres, etc.)
- Use separate AWS accounts for production and non-production environments

## Networking

- Use VPC with private subnets for workloads, public subnets only for load balancers
- Enable VPC Flow Logs for network visibility
- Use Security Groups as the primary network control — deny by default
- Use VPC endpoints for AWS service access to avoid public internet

## Security

- Enable encryption at rest (KMS) and in transit (TLS) for all services
- Never hardcode credentials — use IAM roles, Secrets Manager, or SSM Parameter Store
- Enable CloudTrail for audit logging in all regions
- Use SCPs (Service Control Policies) to enforce guardrails across accounts

## Compute

- Prefer serverless (Lambda, Fargate) for variable workloads
- Use Auto Scaling Groups with health checks for EC2 workloads
- Right-size instances — start small, scale based on metrics
- Use Spot instances for fault-tolerant batch workloads

## Storage & Data

- Enable versioning on S3 buckets; block public access by default
- Use DynamoDB for key-value workloads, RDS/Aurora for relational
- Set lifecycle policies for logs and temporary data
- Use cross-region replication for critical data

## Observability

- Use CloudWatch for metrics, logs, and alarms
- Set up alarms for key metrics: error rates, latency p99, queue depth
- Use X-Ray or OpenTelemetry for distributed tracing
- Centralize logs with CloudWatch Logs Insights or a log aggregator

## Quality Gates

Run these before applying infrastructure changes:

1. `terraform fmt -check` — ensure consistent formatting
2. `terraform validate` — check configuration syntax
3. `terraform plan` — review changes before applying
4. `tfsec .` or `checkov -d .` — scan for security misconfigurations

If any gate fails, fix the issue before proceeding.

## AWS CLI Usage

Use `aws` CLI only when Terraform is not the right tool:

- **Debugging**: `aws logs tail`, `aws ecs describe-tasks`, `aws lambda invoke`
- **Incident response**: `aws ec2 revoke-security-group-ingress`, `aws iam delete-access-key`
- **One-off queries**: `aws s3 ls`, `aws dynamodb scan`, `aws sts get-caller-identity`
- **Operations**: `aws ecs update-service --force-new-deployment`, `aws ssm start-session`

Never use `aws` CLI to create or modify infrastructure that should be in Terraform state.

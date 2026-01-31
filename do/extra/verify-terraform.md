Verify all Terraform module sources in this codebase use the S3 format:
  s3::https://s3-eu-west-1.amazonaws.com/datadeft-tf-modules/{components,services}/<name>-v<version>.zip

  Check:
  1. All `source = "..."` in *.tf files (excluding .terraform/)
  2. Flag any local paths, git URLs, or registry references
  3. Flag missing version suffixes (e.g., -v0.9.1)
  4. Report violations with file:line and the invalid source value

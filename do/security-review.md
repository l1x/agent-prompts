```yaml
kind: prompt
name: security-review
description: "Threat model and security review with actionable findings and remediation"
inputs:
  - name: mission
    required: true
  - name: worktree_path
    required: true
  - name: context
    required: false
    default: ""
outputs:
  - name: findings
    format: json
```

## Mission

{{mission}}

## Instructions

1. Read the code and architecture in `{{worktree_path}}`
2. Apply a threat-centric mindset — assume a hostile environment and search for attack vectors
3. Model threats using STRIDE (Spoofing, Tampering, Repudiation, Information Disclosure, Denial of Service, Elevation of Privilege)
4. Evaluate each component against these principles:
   - **Defense in depth** — never rely on a single control
   - **Least privilege** — minimum permissions for every component, user, and service
   - **Secure defaults** — the easiest path should be the most secure path
   - **Backend enforcement** — never rely on client-side security properties
5. Check for common vulnerabilities:
   - OWASP Top 10 (injection, XSS, broken auth, SSRF, etc.)
   - Hardcoded secrets or credentials
   - Overly permissive IAM roles or network policies
   - Missing input validation at system boundaries
   - Race conditions and TOCTOU bugs
   - Insecure deserialization
   - Missing or weak encryption (at rest and in transit)

## Output

Your output MUST be valid JSON:

```json
{
  "threat_model": "Brief description of the threat landscape for this system",
  "findings": [
    {
      "severity": "critical | high | medium | low",
      "category": "STRIDE category",
      "title": "Short title",
      "description": "What the vulnerability is",
      "attack_vector": "How an attacker would exploit this",
      "impact": "Business impact if exploited",
      "location": "file:line or component name",
      "remediation": "Specific fix — config change, code snippet, or architectural modification"
    }
  ],
  "summary": "Overall security posture assessment"
}
```

## Constraints

- Be specific and actionable — no generic check-the-box advice
- Classify by severity with business impact justification
- Provide concrete remediation (config changes, code snippets, architectural modifications)
- Focus on the risks within this specific context, not theoretical attacks
- Every finding must include a practical remediation
- Do not rewrite the code — only review it

## Context from prior steps

{{context}}

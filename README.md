# Newsletter

[![Version](https://img.shields.io/badge/rustc-1.46+-ab6000.svg)](https://blog.rust-lang.org/2020/03/12/Rust-1.46.html)
[![Actions Status](https://github.com/angelocatalani/newsletter/actions/workflows/ci_and_cd.yml/badge.svg)](https://github.com/angelocatalani/newsletter/actions)
[![Actions Status](https://github.com/angelocatalani/newsletter/actions/workflows/audit.yml/badge.svg)](https://github.com/angelocatalani/newsletter/actions)
[![Actions Status](https://github.com/angelocatalani/newsletter/actions/workflows/scheduled_deploy.yml/badge.svg)](https://github.com/angelocatalani/newsletter/actions)
[![Actions Status](https://github.com/angelocatalani/newsletter/actions/workflows/scheduled_audit.yml/badge.svg)](https://github.com/angelocatalani/newsletter/actions)

Implementation with some personal additions, of the web application described in this amazing
book: [Zero To Production In Rust](https://www.zero2prod.com/index.html?country=Italy&discount_code=VAT20).

The application exposes the following routes until 20 July 2021 (when the DigitalOcean promo credit expires).

```shell
# the health check
curl https://newsletter-5nmom.ondigitalocean.app/health_check
```

```shell
# pending subscription
curl -vv -X POST https://newsletter-5nmom.ondigitalocean.app/subscriptions -d "name=alan%20turing&email=alan_turing%40apple.com"
```

```shell
# confirm subscription
curl -vv https://newsletter-5nmom.ondigitalocean.app/subscriptions/confirm?subscription_token=random-id-sent-by-email
```

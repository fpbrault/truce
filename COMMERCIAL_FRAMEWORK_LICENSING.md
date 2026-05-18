# Framework Licensing

truce ships under a dual-license model with **one specific
restriction**: redistributing truce as part of a **commercial
audio-plug-in framework** for third-party developers requires
written permission. Everything else — including free, open-source
frameworks built on top of truce — runs under the standard
**Apache License, Version 2.0** (`LICENSE-APACHE`) with no extra
hoops.

If you're not commercializing a framework, you almost certainly
don't need to read this document. Build your plug-in, ship your
plug-in, charge whatever you want for your plug-in. You're done.

If you _are_ commercializing a framework on top of truce, the rest
of this doc tells you what counts, what the permission process
looks like, and what terms a grant might carry.

## TL;DR

- Building, shipping, and selling **audio plug-ins or end-user
  audio software** → **Apache License 2.0, free, ship it.** You
  get full Apache 2.0 rights: commercial use, closed-source
  distribution, patent grant, the works. **No restriction
  whatsoever** — this is true even when your plug-in is a paid
  commercial product.
- Building a **free, OSI-licensed framework** on top of truce —
  source fully public, free of charge, no paid offerings tied to
  it → **also Apache 2.0, free, ship it, no permission needed.**
  See the Section 2.1 exemption in `LICENSE`. Voluntary
  sponsorship / patronage is explicitly allowed. We'd love to
  know about it, but it's not required.
- Building a **commercial framework** for plug-in authors —
  anything sold, subscription-gated, dual-licensed commercially,
  or bundled into a paid offering → **Framework License
  required.** Read on.

The gate is specifically on **commercializing a framework on top
of truce**. It is not on commercial use of truce in general.
Plug-ins built with truce can be as commercial as you like; that's
not what this is about.

## What needs a Framework License (commercial framework products only)

The license text defines a "Framework Product" as something that
meets both of:

1. it is offered, distributed, sublicensed, or otherwise provided
   to third-party developers as a means of building audio plug-ins
   or DAW host integrations, **and**
2. the third-party developer's use of the product results in audio
   software that the third-party developer distributes to end
   users or to further developers.

A Framework License is required only when both of those are true
**and** the Framework Product is a **commercial** offering —
i.e. when it doesn't qualify for the Section 2.1 free-OSS
exemption.

Some concrete shapes:

| Use | Side |
|---|---|
| You're building a synth or effect plug-in. It happens to be commercial / closed-source / shipping on the App Store. | Author (no restriction) |
| You're shipping a suite of 30 commercial plug-ins all built on truce. | Author (no restriction) |
| You vendored truce into your build and modified it locally for your plug-in's needs. | Author (no restriction) |
| You're publishing a wrapper crate that smooths some truce API for your own plug-ins (and others find it useful as a transitive dep). | Author (no restriction) |
| You're a hardware vendor whose hardware ships a DSP SDK that internally uses truce — but your SDK's developer-facing surface is your own. | Author (no restriction) |
| You're publishing an OSS framework on top of truce — full source on GitHub under MIT/Apache/MPL, free of charge, no paid tier, accepting voluntary sponsorship donations. | Free-OSS exemption (no restriction) |
| You're publishing an OSS framework on top of truce, but the project also sells "Pro" support contracts or training certifications. | **Commercial Framework — permission required** |
| You're publishing an OSS framework on top of truce, AND dual-licensing it commercially. | **Commercial Framework — permission required** |
| You're publishing a commercial framework on top of truce — closed-source, subscription-gated, or sold as a SaaS. | **Commercial Framework — permission required** |
| You're white-labeling truce as your own plug-in framework offering, regardless of source-availability, regardless of price. | **Commercial Framework — permission required** |

If your situation doesn't fit any of these and you're not sure,
ask — it costs nothing to send the email.

## What the Section 2.1 exemption is for

The exemption exists so OSS community frameworks, academic
projects, research libraries, and hobbyist developer tools on top
of truce can ship without bureaucracy. The litmus test is
**whether money changes hands tied to the framework itself**.

Three rules of thumb for the exemption:

1. **Source must be fully public** under an OSI-approved license.
   "Source-available" licenses (Polyform, BUSL, FSL) don't count.
2. **The framework itself must be free of charge.** Voluntary
   sponsorship / donations are fine; paid tiers / subscriptions /
   commercial dual-licensing are not.
3. **Don't bundle the framework into a paid offering** where the
   framework is the thing being sold. Plug-ins built _with_ the
   framework that are sold by their plug-in authors are obviously
   fine — that's exactly the Section 1 grant.

If a project starts in the exemption and later goes commercial,
the exemption stops applying from that date; existing plug-ins
keep what they already shipped, but the framework's continued
distribution then needs a Framework License under Section 2.

## How to request a Framework License

Send an email to **`framework-licensing@truce.audio`** (or open a
private discussion on the truce-audio GitHub org and tag the
maintainers if email is inconvenient) with:

1. **What you're building.** A paragraph or two. What's the
   product, who is the developer audience, what do they get when
   they use it?
2. **Truce's role.** What is truce doing inside your product? Is
   it the core, a backend among many, an internal implementation
   detail surfaced to your users, something else?
3. **Distribution model.** How does your product reach your
   developers? What's commercial about it — that's what we need
   to understand to size the conversation.
4. **What you'd like the license to look like.** Propose terms —
   even rough ones. (Attribution? Time-limited? Royalty? Free?)
   This makes the conversation faster than starting from zero.

Expect a response **within 2–4 weeks**. The maintainers commit to
acknowledging every well-formed request within that window. We do
not commit to granting any specific request, and we reserve the
right to deny without detailed reasoning.

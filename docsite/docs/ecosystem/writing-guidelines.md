# Writing Guidelines

This page is a temporary baseline for writing and revising Komodo documentation. It exists to keep
the docs grounded in established technical writing guidance, especially when drafting with AI
assistance.

The aim is not stylistic purity. The aim is to produce documentation that is clear, technically
reliable, and readable by someone who is trying to learn the system rather than skim for a single
command.

## Prompt Pack

Use this block when prompting AI to draft or revise docs.

```text
Write Komodo docs in technical-doc style, not marketing, coaching, or chat.
Mode first: explanation, setup, how-to, or reference. Keep one dominant mode per page.
Front-load the point in sentences and paragraphs. Name the actor. Prefer active voice and present
tense unless accuracy requires otherwise.
Use fewer, denser sections. Headings should mark real topic clusters, not micro-points.
Use prose for relationships, causality, and tradeoffs. Use lists for steps, fields, options, and
enumerations.
Keep paragraph cohesion: one idea per paragraph, but let it develop. Do not atomize explanation
into one-sentence fragments by default.
Give each sentence one main burden. Do not combine explanation, qualification, anticipation of
confusion, and navigation in the same sentence unless necessary.
Prefer concrete nouns, real paths, real defaults, real examples, and explicit tradeoffs. Avoid
vague praise words and invented abstractions.
Cut filler and signposting: to be clear, that said, in practice, it is worth noting, key point,
the goal of this page is, first thing to understand, other resources become relevant.
Do not narrate the page: this page covers, use this page when, why this page exists, what this
page does. Explain the system instead.
Do not use unearned ranking or recommendation words: simplest, easiest, best, fastest, safest, for
most users, usually, practical, cleanest, straightforward, natural choice. State the tradeoff
instead.
In explanation and reference pages, do not stage the reader through time with start with, next,
later, after that, becomes relevant once, or once the basic model is in place.
Avoid AI contrast templates: it's not X, it's Y; not just X, but Y; the real issue is.
Avoid punctuation-led style signaling: repeated em dashes, -- pivots, colon scaffolds like Why: or
Key point:.
Do not restate the same point in adjacent paragraphs at different temperatures.
Do not over-introduce or over-summarize. Avoid mini-intros before every list and mini-summaries
after every section unless they add meaning.
Link cleanly to deeper pages where detail belongs. Do not add bridge prose that explains the link
more than the concept.
Verify claims, defaults, recommendations, and examples against source, docs, issues, discussions,
and real behavior.
If the draft feels polished but low-signal, merge sections, cut repetition, replace bullets with
prose where needed, and add concrete detail.
```

## Compaction Pack

Use this compressed version when context is tight and the full page will not survive compaction.

```text
Mode-first. One page one job. Front-load. Active voice. Present tense. Actor named. High section
density, low heading granularity. Prose for causality/tradeoffs; lists only for steps/fields/enums.
One idea per paragraph, one burden per sentence. No paragraph atomization. No taxonomy dumps. No
teaching-order commentary. No temporal staging in non-procedural pages. No page narration. No
self-justifying sections. No link-hub prose. No throat-clearing. No AI pivots or contrast
templates. No em-dash/--/colon addiction. No echo, no intro-summary padding, no over-signposting.
No unearned best/easiest/usually language. Concrete nouns, real paths, real defaults, explicit
tradeoffs. Plain language, no idioms, no vague praise, no invented abstractions. Verify against
source + docs + issues + behavior. If smooth but low-signal: merge, cut 20%, restore prose
continuity, add specifics.
```

## Fast Rules

Use this section as the default pass for every doc edit and review.

- **Mode**: Write to the page's main job. Explain in explanation pages. Guide in setup pages.
  Describe in reference pages. Do not mix modes without reason.
- **Front-load**: Put the key point early in the sentence and paragraph. Do not hide the real point
  at the end.
- **Voice**: Name the actor and prefer active voice. Do not let passive phrasing hide who does
  what.
- **Density**: Prefer fewer, denser sections. Do not create headings for every minor point.
- **Prose**: Use prose for relationships, causality, and tradeoffs. Do not turn every explanation
  into bullets.
- **Lists**: Use lists for steps, fields, options, and enumerations. Do not use lists as the default
  explanation format.
- **Paragraphs**: Let a paragraph develop one idea. Do not atomize the page into one-sentence
  fragments unless emphasis requires it.
- **Directness**: Say the thing once, clearly. Do not add throat-clearing like `to be clear`, `that
  said`, `in practice`, or `it is worth noting` unless it changes meaning.
- **One job per sentence**: Let sentences carry one main burden. Do not combine explanation,
  qualification, anticipation of confusion, and navigation in the same sentence unless the page
  genuinely needs it.
- **No page narration**: Do not narrate the document with lines like `this page covers`, `use this
  page when`, or `what this page does`. Explain the subject directly.
- **Structure**: Use headings to organize conceptual clusters. Do not stack multiple headings with
  one sentence under each.
- **Headings**: Use specific statement or question headings when they help orientation. Avoid vague
  labels like `Overview`, `General`, or `Scope` unless they are genuinely accurate.
- **Specificity**: Prefer concrete nouns, real examples, and explicit tradeoffs. Do not rely on
  vague terms like `powerful`, `flexible`, `robust`, or `meaningful`.
- **Terms**: Use the plainest language that stays technically correct. Avoid idioms, ambiguous
  labels, and invented abstractions when a concrete term exists.
- **Scope**: In explanation pages, cover the first-order concepts first. Do not overload the first
  pass with full taxonomy, edge cases, or secondary resource inventory.
- **No taxonomy dump**: Do not drop a schema or resource catalog into an explanation page unless
  the page is actually about that catalog.
- **Recommendation words**: Do not use `simplest`, `easiest`, `best`, `fastest`, `safest`,
  `usually`, or `for most users` unless the page states the concrete property that earns the claim.
- **Tone**: Be clear, direct, and neutral. Do not sound chatty, congratulatory, overly balanced, or
  customer-service-polished.
- **Punctuation**: Use punctuation for clarity, not style signaling. Do not lean on repeated em
  dashes, `--`, or colon-heavy scaffolding like `Why:` or `Key point:`.
- **Anti-template**: State the point directly. Do not overuse templates like `it's not X, it's Y`,
  `not just X, but Y`, or `the real issue is`.
- **No teaching-order commentary**: Explain the concept itself. Do not narrate the learning order
  with lines like `the first thing to understand` or `other resources become relevant`.
- **No temporal staging**: In explanation and reference pages, do not frame the topic as `start
  with`, `later`, `next`, `after that`, or `once the model is in place`. Use that only in actual
  procedures.
- **No echo**: Advance the explanation. Do not restate the same point in three adjacent paragraphs
  at different temperatures.
- **No padding**: Cut introductions and summaries that only announce what the section will say or
  just said.
- **No self-justification**: Do not add sections like `Why This Page Exists` unless the existence
  of the page itself is the thing being documented.
- **Link cleanly**: Link out where detail belongs. Do not add bridge prose that explains the
  existence of the link more than the target concept itself.
- **Verify**: Check recommendations and defaults against source, docs, issues, and real behavior. Do
  not let polished wording hide uncertainty.
- **AI review**: If the draft feels smooth but low-signal, merge sections, restore prose continuity,
  cut filler, and add concrete detail.

## AI Slop Sweep

Use this as a final pass after the content is technically correct.

Read the page once for structure, then once for sentence-level tells.

Cut or rewrite these patterns aggressively:

- teaching-order commentary such as `the first thing to understand` or `other resources become
  relevant`
- page-narration such as `this page covers`, `use this page when`, or `the rest of this page`
- self-justifying page text such as `why this page exists` when the page body already shows that
  through distinct content
- low-evidence recommendation language such as `for most users`, `usually`, `the cleanest`, or
  `the safest pattern` when the page has not justified the claim
- ranking language such as `simplest`, `easiest`, `best`, or `fastest` when the tradeoff is not
  stated explicitly
- router prose such as `for the full details, see ...` when a normal inline link would do
- abstract labels that hide a simpler term
- repeated “helpful” framing around the same point

When sweeping a draft, ask:

1. Which sentences are explaining the system?
2. Which sentences are explaining the document, the learning order, or the navigation?
3. Which recommendation words are earned by evidence on this page, and which are just tone?
4. Which lists are clarifying, and which are standing in for missing prose?

The usual fixes are:

- replace page narration with direct explanation
- replace soft recommendation language with a direct statement of the tradeoff
- replace ranking words with the concrete property that matters, such as fewer services, fewer
  mounts, direct host access, or less manual key management
- cut one of two adjacent sentences that do the same work
- replace abstract wording with the concrete host, file, command, or resource name

## Why This Exists

AI-assisted drafting tends to fail in recognizable ways:

- too many headings with too little content under each one
- too many short paragraphs that do not develop an idea
- too many lists where prose would explain relationships better
- generic transitions and connective filler
- onboarding language that sounds encouraging but adds little information
- teaching-order commentary that narrates what the reader should understand first instead of
  explaining the thing
- taxonomy dumps that name a lot of concepts without helping the reader relate them
- link-hub prose that talks about where to click instead of progressing the explanation
- confident wording around points that are still ambiguous or unverified

Those patterns make documentation feel polished while weakening its value. This page is intended to
counter that tendency.

## Reader Model

The primary question is not whether a page is easy to scan. It is whether the page supports the
reader's actual mode of use.

Some Komodo pages are read transactionally. A user wants a command, a setting, or a field
definition. Those pages benefit from high scannability and compact structure.

Other pages are read for understanding. A user is trying to learn what Core and Periphery do, what
runs where, or how to choose between several setup paths. Those pages need stronger continuity of
exposition. They benefit from clear headings and emphasis, but they also need prose that develops an
idea across a section.

This distinction is supported by several style guides:

- Google's guidance on [organizing large
  documents](https://developers.google.com/tech-writing/two/large-docs) emphasizes logical
  development and progressive disclosure.
- Google's guidance on [paragraph
  structure](https://developers.google.com/style/paragraph-structure) says each paragraph should
  address one idea, but should not be split apart if that one idea still belongs together.
- Digital.gov's guidance on [writing for understanding](https://digital.gov/guides/plain-language/writing/)
  favors shorter words, short sections, active voice, and present tense.
- GOV.UK's guidance on [writing for user
  interfaces](https://www.gov.uk/service-manual/design/writing-for-user-interfaces) emphasizes short
  and direct copy for transactional content, which is a useful reminder that not every page has the
  same reading mode.

## Core Principles

### Prefer clarity over apparent polish

The writing should make the system easier to understand, not merely appear smooth or professional.
If a sentence sounds polished but adds little information, cut it.

### Write to the dominant user need

Each page should have a primary mode. A page explaining the architecture should primarily explain. A
page helping a reader choose between setup paths can be more directive. A reference page should stay
factual and descriptive.

Diataxis is useful here as a lightweight check, not as a full information architecture. If a page is
trying to explain but keeps slipping into coaching, or trying to guide but keeps drifting into
background theory, rewrite toward the dominant mode.

Useful references:

- [Diataxis: Start here](https://diataxis.fr/start-here/)
- [Diataxis: The compass](https://diataxis.fr/compass/)

### Front-load the point

The reader should not have to wait for the real point of a sentence or paragraph. Put the critical
information first, then expand it.

This comes through clearly in Google's guidance on [paragraph
structure](https://developers.google.com/style/paragraph-structure), which recommends putting
critical information first in a paragraph, and GOV.UK's guidance on [writing for user
interfaces](https://www.gov.uk/service-manual/design/writing-for-user-interfaces), which recommends
putting important words first and dropping unnecessary words.

### Name the actor

When a sentence describes an action, say who performs it. This matters a lot in Komodo docs because
the difference between Core, Periphery, the database, and the host runtime is often the entire
point.

Digital.gov's guidance on [writing for understanding](https://digital.gov/guides/plain-language/writing/)
explicitly favors active voice because it makes clear who does what. Use passive voice only when it
is genuinely the clearest choice.

### Prefer accumulated explanation over fragmentation

Breaking content into smaller units can improve scanability. Overdoing it makes the page choppy and
disjointed. A section should carry enough content to justify existing as a section.

Microsoft's guidance on
[headings](https://learn.microsoft.com/en-us/style-guide/scannable-content/headings) is useful here:
headings should be used judiciously, and each heading should introduce a distinct topic. If several
adjacent headings each carry only a narrow claim, they likely belong in a single larger section.

## Headings

Headings are useful. They help the reader locate concepts and re-enter the page after scrolling. The
problem is not using headings. The problem is using them at too fine a granularity.

Use headings to mark conceptual clusters:

- architecture and execution
- resource model
- topology choices
- deployment options

Avoid headings that isolate every small point. When a heading is followed by one sentence or a tiny
bullet list, consider whether that content belongs in the surrounding section instead.

Grounding:

- Microsoft: [Headings](https://learn.microsoft.com/en-us/style-guide/scannable-content/headings)
- Google: [Organizing large documents](https://developers.google.com/tech-writing/two/large-docs)

## Paragraphs

Paragraphs should carry one idea, but that idea often needs more than one sentence. A page full of
one-sentence paragraphs reads like notes, not documentation.

Single-sentence paragraphs are valid when they do real work:

- a direct summary sentence at the start of a section
- a deliberate emphasis point
- a short transition to a list or table

They should not become the default shape of the page.

Google's [paragraph structure](https://developers.google.com/style/paragraph-structure) guidance is
the most useful baseline here:

- one idea per paragraph
- put critical information first
- do not split a paragraph merely to make it shorter if the idea is still one idea

For Komodo docs, that usually means explanatory pages should favor medium-length paragraphs that
define a concept, explain why it matters, and connect it to the next idea.

## Lists

Lists are for material that benefits from enumeration:

- prerequisites
- fields
- options
- ordered steps
- short comparisons

Lists are not the default format for explanation. If the main value of a section is the relationship
between ideas, prose usually works better.

Microsoft's guidance on
[lists](https://learn.microsoft.com/en-us/style-guide/scannable-content/lists) is a useful
constraint:

- keep list items structurally consistent
- keep them reasonably short
- make sure the purpose of the list is clear

One especially useful point from Microsoft is to avoid list items that merely complete a hanging
introductory fragment. That is relevant to localization, but it also tends to produce weak rhythm in
general prose.

## Tone

The baseline tone for Komodo docs should be:

- clear
- direct
- technically precise
- neutral

It should not be:

- chatty
- congratulatory
- overly familiar
- padded with encouragement or generic reassurance

GOV.UK's guidance is especially useful here. It recommends copy that is short and direct,
approachable but not overly familiar, and notes that if users do not notice the copy, that is often
a sign the writing is doing its job.

References:

- GOV.UK: [Writing for user
  interfaces](https://www.gov.uk/service-manual/design/writing-for-user-interfaces)
- GOV.UK: [Technical content style
  guide](https://www.gov.uk/guidance/style-guide/technical-content-a-to-z)

For Komodo docs, this translates into a few practical rules:

- avoid phrases like `simply`, `just`, `obviously`, `easy`, or `of course`
- avoid filler such as `the goal of this page is to`, unless the scope of the page genuinely needs
  to be established
- avoid sentences that restate the heading without adding new information
- avoid product-marketing cadence in technical pages

## Typical AI Drafting Problems

The following problems are common in AI-generated drafts and should be reviewed aggressively.

### Over-chunking

The page has many sections, but each section contains too little information. This creates a
stop-start reading experience and weakens continuity.

### List-heavy exposition

The page explains concepts by turning everything into bullets. The reader can scan it quickly, but
comes away with disconnected facts rather than a coherent model.

### Instructional padding

The prose contains transitions and encouragement that sound supportive but do not add information.
Typical examples include phrases like `for most users`, `a good starting point`, or `this gives you
the fastest path` when the page does not need that layer of commentary.

### Restatement without development

A section opens with a heading, then repeats the same idea in slightly different words without
moving the explanation forward.

### Synthetic confidence

The draft presents ambiguous choices as settled best practice without enough evidence. This is
especially risky in setup and architecture docs.

### Teaching-order commentary

The draft narrates what the reader should learn first instead of explaining the concept itself.
Typical examples include lines like `the first thing to understand is`, `other resources become
relevant once`, or `this is the main thing to know`.

These sentences often feel helpful while adding little substance. They are usually better replaced
with a direct statement of the concept.

### Taxonomy dumps

The draft names a set of resources, features, or moving parts without giving the reader enough
structure to relate them. This often happens when an explanation page slips into schema listing.

The symptom is high noun density with weak conceptual payoff. A short descriptive sentence per item
can help, but often the real fix is to cut the list down to first-order concepts.

### Link-hub prose

The draft keeps interrupting itself to explain where detail lives elsewhere. This usually appears as
bridge prose such as `for more detail, see ...` or `if you need setup details, see ...`.

Cross-links are good. The problem is when navigation displaces explanation. Link the concept where
it appears, then move on.

### Invented abstraction

The draft prefers abstract labels over the concrete words the user would naturally reach for. Terms
like `execution target`, `upstream path`, or `workflow shape` can be valid, but they often hide a
simpler phrase such as `host where work runs`, `default database option`, or `common setup pattern`.

This is usually a plain-language failure. Replace the abstraction unless it is doing real technical
work.

## Common AI Writing Markers

No single marker proves that a passage was written by AI. Human writers can produce every pattern in
this list. The problem is clustering: when several of these signals appear together, the draft often
needs substantial rewriting.

The value of this section is practical rather than forensic. These are useful review markers because
they correlate with low-signal, over-smoothed prose.

### Em dash overuse

One of the clearest modern markers is excessive use of em dashes as the default way to introduce
contrast, clarification, or rhythm.

The problem is not the punctuation mark itself. It is repetitive deployment. When every paragraph
contains an em dash, the prose starts to sound synthetic and over-managed.

References:

- REM Web Solutions: [Excessive Use of Em
  Dashes](https://www.remwebsolutions.com/blog/identifying-ai-writing)
- Intender: [Excessive use of horizontal lines /
  dashes](https://intender.com.au/ai-content-detection-signs/)

### Template contrast framing

AI-assisted drafts often lean on stock contrast templates:

- `it's not X, it's Y`
- `not just X, but Y`
- `the real issue is not X`
- `rather than X, think of it as Y`

Used occasionally, these are normal. Used repeatedly, they flatten the prose into a series of
rhetorical pivots.

These patterns usually indicate that the draft is spending too much effort on framing and not enough
on direct explanation. GOV.UK's preference for short, direct copy is a useful corrective here.

Reference:

- GOV.UK: [Keep copy short and
  direct](https://www.gov.uk/service-manual/design/writing-for-user-interfaces)

### Connective filler and throat-clearing

Common examples:

- `to be clear`
- `to be more precise`
- `that said`
- `with that in mind`
- `the key point is`
- `in practice`
- `it is worth noting`

These phrases are not wrong. The problem is overuse. They often appear when the draft is trying to
sound careful, balanced, or well-signposted while adding little information.

This is closely related to instructional padding. If the sentence still works without the phrase,
remove it.

References:

- Google: [Paragraph structure](https://developers.google.com/style/paragraph-structure)
- GOV.UK: [Keep copy short and
  direct](https://www.gov.uk/service-manual/design/writing-for-user-interfaces)

### Repetition at different temperatures

AI drafts often restate the same idea multiple times with slight tonal changes:

- plain statement
- clarified restatement
- balanced synthesis

The result feels thorough but does not materially advance the explanation.

This is one of the easiest ways for a document to become longer and weaker at the same time. When
reviewing, check whether adjacent paragraphs differ in substance or only in phrasing.

Related guidance:

- Google: [Organizing large documents](https://developers.google.com/tech-writing/two/large-docs)
- Google: [Paragraph structure](https://developers.google.com/style/paragraph-structure)

### Even rhythm and symmetrical sentence patterns

AI-generated text often has unusually even cadence:

- similar sentence lengths
- repeated sentence openings
- highly balanced paragraph shapes

Human prose usually has more variation in rhythm and emphasis.

This is not a strict quality rule. Some formal docs should be regular. But when an explanatory page
becomes too rhythmically uniform, it often feels synthetic and low-energy.

References:

- Intender: [Perfect sentence length](https://intender.com.au/ai-content-detection-signs/)
- REM Web Solutions: [Monotonous rhythm and flat
  tone](https://www.remwebsolutions.com/blog/identifying-ai-writing)

### Excessive neutrality or servile positivity

AI drafts often default to a tone that is:

- highly balanced
- careful to the point of vagueness
- relentlessly polite
- reluctant to make a crisp claim

This can produce what reads like customer-service prose rather than technical documentation.

For docs, the better standard is neutral and direct, not endlessly conciliatory.

References:

- REM Web Solutions: [Servile positivity and overly polite
  statements](https://www.remwebsolutions.com/blog/identifying-ai-writing)
- GOV.UK: [Tone](https://www.gov.uk/service-manual/design/writing-for-user-interfaces)

### Generic examples and soft abstraction

AI text often claims to explain with examples while remaining vague:

- no concrete filenames, hosts, or commands
- no real topology details
- no explicit tradeoffs
- generic terms such as `modern`, `flexible`, `powerful`, `robust`, `meaningful`

This is usually a sign that the draft has the shape of explanation without the substance of
explanation.

References:

- Pencil: [Generic or vague
  content](https://help.trypencil.com/en/articles/11027140-what-are-the-key-signs-of-ai-generated-copy)
- Intender: [Generic examples and lack of
  specificity](https://intender.com.au/ai-content-detection-signs/)

Related baseline guidance:

- GOV.UK: [Technical content style
  guide](https://www.gov.uk/guidance/style-guide/technical-content-a-to-z)
- Digital.gov: [Writing for understanding](https://digital.gov/guides/plain-language/writing/)

### Heading inflation

AI-generated docs often create too many headings and too little content under each one. This creates
the appearance of organization without enough internal development.

This is one of the main failure modes in conceptual documentation. It improves surface scanability
while weakening continuity.

References:

- Microsoft: [Headings](https://learn.microsoft.com/en-us/style-guide/scannable-content/headings)
- Google: [Organizing large documents](https://developers.google.com/tech-writing/two/large-docs)

### List fixation

AI models are very willing to turn any explanation into:

- a numbered list
- a bulleted list
- a set of parallel headings

Lists are useful, but they are not a substitute for exposition. If a section is mainly about
relationships or causality, rewrite it as prose.

References:

- Microsoft: [Lists](https://learn.microsoft.com/en-us/style-guide/scannable-content/lists)
- Google: [Organizing large documents](https://developers.google.com/tech-writing/two/large-docs)

### Over-introducing and over-summarizing

AI drafts often add:

- a mini-introduction before every list
- a mini-summary after every section
- a closing synthesis even when the page does not need one

These patterns create friction because the page keeps announcing what it is about to say and what it
just said instead of saying the thing once, clearly.

This is one of the easiest markers to cut during editing.

Related guidance:

- Google: [Introduce a document](https://developers.google.com/tech-writing/two/large-docs)
- Microsoft: [Introductory text for
  lists](https://learn.microsoft.com/en-us/style-guide/scannable-content/lists)

### Meta-teaching and temporal framing

Another recurrent marker is narration about the learning sequence:

- `the first resource to understand`
- `once the basic model is in place`
- `the next thing to know`
- `this page helps you understand`

This pattern often shows up when the draft is trying to sound instructional without deciding whether
it is writing explanation or tutorial content. It is closely related to mode drift.

The useful question is whether the sentence teaches the system or comments on the act of teaching.
If it is the latter, cut it.

Related baseline guidance:

- Google: [Organizing large documents](https://developers.google.com/tech-writing/two/large-docs)
- Digital.gov: [Writing for understanding](https://digital.gov/guides/plain-language/writing/)

### Link-hub prose and navigation overgrowth

AI drafts often insert link narration into the body text:

- `for more detail, see`
- `if you need setup details, see`
- `this is covered on`

These phrases are not wrong, but repeated use makes the prose sound like a router instead of a
document. This is especially common after AI has already over-split the information architecture and
starts compensating with navigation text.

Related baseline guidance:

- Google: [Organizing large documents](https://developers.google.com/tech-writing/two/large-docs)
- Digital.gov: [Headings provide structure and orient the
  reader](https://digital.gov/guides/plain-language/design/headings)

### Temporal or staged explanation in non-procedural pages

Another current failure mode is talking about concepts as if the reader is moving through a lesson:

- `start with`
- `later`
- `next`
- `after that`
- `becomes relevant once`

That is fine in tutorials and procedures. It is often wrong in explanation or reference pages,
where the reader needs a description of the system, not narration about a learning sequence.

This pattern often co-occurs with teaching-order commentary and is worth cutting aggressively.

### Flawless but bloodless copy

Another common marker is prose that is mechanically clean but stylistically sterile:

- no rough edges
- no compression
- no distinctive emphasis
- no local context

Brandeis includes overly flawless writing and repetitive formulaic structures in its list of common
indications of AI use. That is useful as a warning, not a diagnosis. If the copy feels perfectly
smooth while saying little, it probably needs harder editing.

Reference:

- Brandeis: [Common indications of AI
  use](https://www.brandeis.edu/student-rights-community-standards/academic-integrity/faculty/common-indications.pdf)

### Unsupported assertions, invented facts, and fake confidence

AI-generated prose can sound authoritative while being wrong, vague, or partly invented. In docs,
this matters more than style.

Any recommendation, comparison, default, or architecture claim should be checked against:

- source code
- existing docs
- issues and discussions
- real setup validation where possible

References:

- REM Web Solutions: [Incorrect or invented
  facts](https://www.remwebsolutions.com/blog/identifying-ai-writing)
- Brandeis: [Factual inaccuracies or logical
  inconsistencies](https://www.brandeis.edu/student-rights-community-standards/academic-integrity/faculty/common-indications.pdf)

## Recent Model-Specific Markers

Model behavior changes frequently. These markers are recent community observations, not stable
rules. They are worth tracking because they often map to real documentation quality problems.

### Claude: `--` as the new em dash

Recent Claude users report that some models now replace the em dash with a double hyphen. This
appears to be a reaction to users treating the em dash as an obvious AI marker.

In practice, the issue is still the same: repeated punctuation-driven contrasts. Whether the draft
uses `—` or `--`, the review question is whether the prose depends too heavily on dramatic pivots
instead of direct statements.

References:

- Reddit: [Anyone else see Claude now uses 2 dashes instead of
  emdash?](https://www.reddit.com/r/ClaudeAI/comments/1rk9kdl/anyone_else_see_claude_now_uses_2_dashes_instead/)
- Reddit: [Sonnet and Opus 4.6 have developed a serious em-dash and colon
  addiction](https://www.reddit.com/r/ClaudeAI/comments/1ra1709/sonnet_and_opus_46_have_developed_a_serious/)

Related baseline guidance:

- GOV.UK: [Keep copy short and
  direct](https://www.gov.uk/service-manual/design/writing-for-user-interfaces)
- Google: [Voice and tone](https://developers.google.com/style/tone)

### Claude: colon-heavy scaffolding

Another recent Claude pattern is colon-heavy framing:

- `Why:`
- `Key point:`
- `In practice:`
- `The tradeoff is:`

These constructions are not always wrong, but repeated use often indicates writing that is
structured around signposting rather than explanation.

This connects directly to broader heading and list guidance. If every paragraph begins by announcing
its function, the draft is usually over-signposted.

References:

- Reddit: [Sonnet and Opus 4.6 have developed a serious em-dash and colon
  addiction](https://www.reddit.com/r/ClaudeAI/comments/1ra1709/sonnet_and_opus_46_have_developed_a_serious/)

Related baseline guidance:

- Digital.gov: [Headings provide structure and orient the
  reader](https://digital.gov/guides/plain-language/design/headings)
- Microsoft: [Headings](https://learn.microsoft.com/en-us/style-guide/scannable-content/headings)

### Claude: clarification tics and safety-balanced phrasing

Recent complaints about Claude-generated prose repeatedly mention phrasing like:

- `I think it's worth noting`
- `to be clear`
- `that said`
- `the key point is`

These phrases often signal a draft that is trying to sound balanced, non-confrontational, and
explicit at every turn. The result is commonly over-qualified prose with too much connective
material.

This is best treated as a tone and concision problem, not a model identity problem.

References:

- Reddit: [I see Claude's writing everywhere and it's starting to feel like an AI
  condom](https://www.reddit.com/r/ClaudeAI/comments/1rjeqg3/i_see_claudes_writing_everywhere_and_its_starting/)

Related baseline guidance:

- GOV.UK: [Tone](https://www.gov.uk/service-manual/design/writing-for-user-interfaces)
- Google: [Voice and tone](https://developers.google.com/style/tone)
- Digital.gov: [Writing for understanding](https://digital.gov/guides/plain-language/writing)

### Claude: tutorial-style over-explanation

Recent Claude Code discussions include users explicitly forbidding tutorial style, over-explanation,
and unsolicited documentation. That lines up with a pattern visible in many AI drafts: the model
tries to be helpful by narrating too much.

In docs work, this usually shows up as:

- explaining obvious transitions
- justifying standard decisions
- adding a mini-lesson where the page only needs one paragraph of context

References:

- Hacker News: [What Claude Code chooses](https://news.ycombinator.com/item?id=47169757)
- Hacker News: [Hey, Boris from the Claude Code team here. A few
  tips](https://news.ycombinator.com/item?id=46256606)

Related baseline guidance:

- Google: [Organizing large documents](https://developers.google.com/tech-writing/two/large-docs)
- Diataxis: [Start here](https://diataxis.fr/start-here/)

### Codex: notes and documentation instead of action

The most recent Codex-specific complaints are less about sentence-level style and more about work
allocation. Users report cases where Codex writes notes, brief updates, or documentation-like
content instead of doing the requested task.

For documentation review, this matters because it creates text that looks useful but substitutes
commentary for execution or explanation.

Reference:

- Reddit: [Codex now writes documents instead of
  coding](https://www.reddit.com/r/codex/comments/1os63ne/codex_now_writes_documents_instead_of_coding/)

Related baseline guidance:

- Google: [Introduce a document](https://developers.google.com/tech-writing/two/large-docs)
- Digital.gov: [Requirements for plain writing](https://digital.gov/resources/plain-writing-act)

### Codex and coding agents: over-abstraction and file proliferation

Recent Codex discussions also mention over-abstraction, too many new files, and an increase in
explanatory material when direct edits were expected.

That is not only a coding workflow issue. In docs, the analogous pattern is over-structuring: too
many sections, too many framing layers, too much machinery around too little substance.

References:

- Reddit: [Codex just got dumb in the last few
  days?](https://www.reddit.com/r/codex/comments/1o26b5r/codex_just_got_dumb_in_the_last_few_days/)
- Hacker News: [Addendum to GPT-5 system card:
  GPT-5-Codex](https://news.ycombinator.com/item?id=45253458)

Related baseline guidance:

- Microsoft: [Headings](https://learn.microsoft.com/en-us/style-guide/scannable-content/headings)
- Google: [Organizing large documents](https://developers.google.com/tech-writing/two/large-docs)

### Practical rule

Do not overfit to current model quirks. The presence or absence of one marker is not important. The
useful question is whether the marker corresponds to a recognized documentation problem:

- em dash or `--` addiction maps to punctuation-led over-signposting
- clarification tics map to filler and tone drift
- tutorializing maps to over-explanation and mode confusion
- notes instead of action map to commentary displacing useful content
- over-abstraction maps to heading inflation and low section density

If the writing is strong, these markers stop mattering. If the writing is weak, they are usually
symptoms rather than root causes.

## Generation And Review Checklist

This checklist can be used either while prompting AI or while reviewing a draft.

### Before generation

- State the page's purpose explicitly: explanation, setup choice, reference, or tutorial.
- State the intended reader and what they are trying to understand.
- Ask for prose-first explanation unless a list is clearly better.
- Ask for fewer, denser sections rather than many micro-sections.
- Ask to avoid filler phrases, rhetorical contrast templates, and over-summary.
- Ask for concrete examples grounded in Komodo's actual concepts and behavior.

### During review

Check for the following:

- repeated em dashes
- repeated contrast templates such as `it's not X, it's Y`
- repeated connective phrases like `that said` or `to be clear`
- three paragraphs in a row saying nearly the same thing
- headings with only one sentence underneath
- lists that should be prose
- generic claims with no concrete detail
- neutral or overly polite tone where a direct statement would be clearer
- unsupported defaults or recommendations

### Rewrite strategies

When one of these patterns appears, the usual fixes are:

- merge adjacent sections
- replace bullets with prose
- cut introductory and concluding filler
- replace rhetorical framing with direct statements
- add concrete detail or remove the claim
- shorten the page by removing repetition, not just trimming sentences
- check whether current model quirks are disguising an older underlying problem such as filler,
  over-signposting, or over-chunking

## Suggested Review Process

When reviewing a draft, ask:

1. What is the reader trying to do or understand on this page?
2. Does the page stay in that mode?
3. Are the headings organizing real conceptual units?
4. Are the paragraphs developing ideas, or merely breaking them apart?
5. Are lists used because they clarify the material, or because they are easy to generate?
6. Does any sentence exist only to sound smooth or friendly?
7. Are claims about recommendations or defaults actually supported?

If a page still feels choppy after copy-editing, the problem is often structural rather than
stylistic. Merge sections, restore prose continuity, and reduce heading granularity before polishing
sentences.

## Vale

[Vale](https://docs.vale.sh/) is a prose linter that can help enforce parts of this baseline.

Vale is useful for:

- inconsistent terminology
- weak phrases and weasel words
- repeated wording
- banned filler terms
- style consistency across multiple authors

Vale is not sufficient for:

- deciding whether a section should exist
- judging whether a page is too fragmented
- deciding whether prose should replace a list
- deciding whether a page is explanation, how-to, or reference

Vale should be treated as a guardrail, not as the source of truth.

Useful Vale references:

- [Vale introduction](https://docs.vale.sh/)
- [Vale packages](https://vale.sh/docs/keys/packages)
- [Vale package explorer](https://vale.sh/explorer/)

Useful packages for this kind of work:

- `Google`
- `Microsoft`
- `write-good`
- `proselint`
- `alex`
- `Readability`

For Komodo-specific docs work, custom Vale rules are likely worthwhile for:

- em dash overuse
- banned filler phrases
- stock contrast templates
- discouraged weasel terms such as `simply` or `easy`
- repetitive heading patterns

## Terms Used In Reviews

The following terms are useful when discussing draft quality:

- **Heading granularity**: how coarse or fine the section structure is
- **Over-chunking**: breaking content into too many small sections
- **Paragraph cohesion**: whether a paragraph develops a single idea clearly
- **List-heavy exposition**: using bullets where prose would explain better
- **Logical development**: whether the page builds understanding in a coherent sequence
- **Instructional padding**: connective text that sounds helpful but adds little information
- **Mode drift**: a page slipping away from its main purpose

## Source Notes

These guidelines draw primarily from:

- Google: [Paragraph structure](https://developers.google.com/style/paragraph-structure)
- Google: [Organizing large documents](https://developers.google.com/tech-writing/two/large-docs)
- Microsoft: [Headings](https://learn.microsoft.com/en-us/style-guide/scannable-content/headings)
- Microsoft: [Lists](https://learn.microsoft.com/en-us/style-guide/scannable-content/lists)
- Digital.gov: [Writing for understanding](https://digital.gov/guides/plain-language/writing/)
- Digital.gov: [Headings](https://digital.gov/guides/plain-language/design/headings)
- GOV.UK: [Writing for user
  interfaces](https://www.gov.uk/service-manual/design/writing-for-user-interfaces)
- GOV.UK: [Technical content style
  guide](https://www.gov.uk/guidance/style-guide/technical-content-a-to-z)
- Vale: [Introduction](https://docs.vale.sh/)

This page is intentionally pragmatic. If a rule makes the docs worse, ignore the rule and improve
the docs.

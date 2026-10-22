---
id: translation-review
---
Compare both Japanese source and Vietnamese target, clause by clause. Check
omission, addition, mistranslation, speaker/pronoun, glossary, honorific, ruby,
number/name and unresolved-source issues. Every issue needs a verbatim source
and/or target span according to its type, a block ID, severity and explanation.
Do not infer fidelity from IDs, word counts, fluency or a numeric score.
Check the current revision only. A missing clause is blocking even if all IDs exist.
Do not lower severity merely to permit export. Unverifiable source remains unresolved.
Return a ruby_decisions entry for every owned ruby annotation, with its index,
base, reading, decision (phonetic, semantic, unresolved), reason and target_quote.
Semantic ruby needs a verbatim target span showing its additional meaning.
Mechanical number/name signals are evidence to assess, not semantic conclusions.

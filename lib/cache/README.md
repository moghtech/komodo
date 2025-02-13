# Cache module

Contains a thread-safe async timeout cache implementation.
Used to cache outputs in memory for a limited time period.
Can be used to avoid re-running the underlying process which generated an output in too short a timeframe.
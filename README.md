# Running L1 & L2 Together: Era + Reth


## Issues:
currently era requires a nightly rust compiler - and the latest one that works, doesn't support simd anymore - so small fixes had to be added to packed_simd and era-boojum.


Also the era_test_harness 1.4.0 was not upgraded with the latest boojum api change. So I had to 'move back' 1.4.1 and boojum to the PRs before the API change.


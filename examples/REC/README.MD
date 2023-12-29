These are various example rewrite systems from the Rewrite Engine Competition
[1]. Note that the ones with META blocks have been omitted. These are used for
comparison studies between mCRL2 and the Sabre rewriter rewritten in Rust. The
benchmark results are shown below (updated 29/12/2023).


| Rewrite System | upstream jitty | upstream jittyc | innermost | sabre |
|           add8 |           1156 |
|          add32 |            571 |
|          add16 |           1250 | 
|            mma |          36246 |
|    benchtree10 |        2076868 |
|    benchtree20 | 



 [1] Durán, F., Garavel, H. (2019). The Rewrite Engines Competitions: A RECtrospective. In: Beyer, D., Huisman, M., Kordon, F., Steffen, B. (eds) Tools and Algorithms for the Construction and Analysis of Systems. TACAS 2019. Lecture Notes in Computer Science(), vol 11429. Springer, Cham. https://doi.org/10.1007/978-3-030-17502-3_6
These are various example rewrite systems from the Rewrite Engine Competition
[1]. Note that the REC specifications with META blocks have been omitted since
it is unclear how to expand them. The resulting mCRL2 specifications are used
for comparison studies between mCRL2 and the Sabre rewriter rewritten in Rust.
The benchmark results are shown below. Timeout of 10 minutes (600 seconds),
indicated by '-'.


| Rewrite System | upstream jitty (4e954e) | upstream jittyc (4e954e) | innermost | sabre |
|           add8 |                    1156 |
|          add16 |                     294 | 
|          add32 |                     571 |
|    benchexpr10 |                       9 |
|    benchexpr20 |                    8421 |
|    benchexpr22 |                   55912 |
|     benchsym10 |                      14 |
|     benchsym20 |                   10799 |
|     benchsym22 |                   30960 |
|    benchtree10 |                       - |
|    benchtree20 |                       - |
|    benchtree22 |                       - |
|   binarysearch |                   58935 |
|   bubblesort10 |                       0 |
|   bubblesort20 |                       0 |
|  bubblesort100 |                      38 |
|  bubblesort720 |                   14221 |
| bubblesort1000 |                   58835 |
|          calls |                       1 |
|         check1 |                       0 |
|         check2 |                       0 |
|     confluence |                       0 |
|        closure |                     431 |
|           dart |                     100 |
|          empty |                       0 |
|       evalexpr |                   12015 |
|        evalsym |                  560323 |
|       evaltree |                   28798 |
|     factorial5 |                       0 |
|     factorial6 |                       1 |
|     factorial7 |                      11 |
|     factorial8 |                      90 |
|     factorial9 |                     736 |
|          fib32 |                   28743 |
|       fibfree  |                       3 |
|    fibonacci05 |                       0 |
|    fibonacci18 |                      17 |
|    fibonacci19 |                      32 |
|    fibonacci20 |                      60 |
|    fibonacci21 |                      66 |
| garbagecollection |                    0 |
|         hanoi4 |                       0 |
|         hanoi8 |                       2 |
|        hanoi12 |                      42 |
|        hanoi16 |                     813 |
|        hanoi20 |                   14807 |
|         intnat |                      16 |
|          merge |                       0 |
|    mergesort10 |                       0 |
|   mergesort100 |                       - |
|  mergesort1000 |                       - |
|  missionaries2 |                       2 |
|  missionaries3 |                       9 |
|            mma |                   36246 | 2008
|           mul8 |                   1074  |
|          mul16 |                    6704 |
|          mul32 |                   22961 |
|        natlist |                      33 |
|       langton6 |                       - |
|       langton7 |                       - |
|         logic3 |                       0 |
|        oddeven |                    1372 |
|  permutations6 |                      36 |
|          omul8 |                   58671 |
|         omul32 |                   22992 |
|          order |                       0 |
|    quicksort10 |                      16 |
|   quicksort100 |                       - |
|  quicksort1000 |                       - |
| searchinconditions |                   0 |
| soundnessofparallelengines |           0 |
|      revnat100 |                       2 |
|     revnat1000 |                     240 |
|    revnat10000 |                   25160 |
|        sieve20 |                       0 |
|       sieve100 |                      25 |
|      sieve1000 |                    5121 |
|      sieve2000 |                   30920 |
|     sieve10000 |                       - |
|          tak18 |                     150 |
|          tak36 |                   12578 |
|  tautologyhard |                       0 |

 [1] Durán, F., Garavel, H. (2019). The Rewrite Engines Competitions: A RECtrospective. In: Beyer, D., Huisman, M., Kordon, F., Steffen, B. (eds) Tools and Algorithms for the Construction and Analysis of Systems. TACAS 2019. Lecture Notes in Computer Science(), vol 11429. Springer, Cham. https://doi.org/10.1007/978-3-030-17502-3_6
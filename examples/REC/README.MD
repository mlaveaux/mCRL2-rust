These are various example rewrite systems from the Rewrite Engine Competition
[1]. Note that the REC specifications with META blocks have been omitted since
it is unclear how to expand them. The resulting mCRL2 specifications are used
for comparison studies between mCRL2 and the Sabre rewriter rewritten in Rust.
The benchmark results are shown below. Timeout of 10 minutes (600 seconds),
indicated by '-'.


| Rewrite System | upstream jitty (4e954e) | upstream jittyc (4e954e) | innermost | sabre |
|           add8 |                    1156 |                       14 |
|          add16 |                     294 |                       22 |
|          add32 |                     571 |                       10 |
|    benchexpr10 |                       9 |                        0 |
|    benchexpr20 |                    8421 |                      445 |
|    benchexpr22 |                   55912 |                     1768 |
|     benchsym10 |                      14 |                        0 |
|     benchsym20 |                   10799 |                      551 |
|     benchsym22 |                   30960 |                     1768 |
|    benchtree10 |                       - |                        - |
|    benchtree20 |                       - |                        - |
|    benchtree22 |                       - |                        - |
|   binarysearch |                   58935 |                     2908 |
|   bubblesort10 |                       0 |                        0 |
|   bubblesort20 |                       0 |                        0 |
|  bubblesort100 |                      38 |                        1 |
|  bubblesort720 |                   14221 |                      374 |
| bubblesort1000 |                   58835 |                     1057 |
|          calls |                       1 |                        0 |
|         check1 |                       0 |                        0 |
|         check2 |                       0 |                        0 |
|     confluence |                       0 |                        0 |
|        closure |                     431 |                       23 |
|           dart |                     100 |                        0 |
|          empty |                       0 |                        0 |
|       evalexpr |                   12015 |                      472 |
|        evalsym |                  560323 |                    16896 |
|       evaltree |                   28798 |                        0 |
|     factorial5 |                       0 |                        0 |
|     factorial6 |                       1 |                        0 |
|     factorial7 |                      11 |                        0 |
|     factorial8 |                      90 |                        4 |
|     factorial9 |                     736 |                       69 |
|          fib32 |                   28743 |                     2713 |
|       fibfree  |                       3 |                        0 |
|    fibonacci05 |                       0 |                        0 |
|    fibonacci18 |                      17 |                        0 |
|    fibonacci19 |                      32 |                        0 |
|    fibonacci20 |                      60 |                        1 |
|    fibonacci21 |                      66 |                        1 |
| garbagecollection |                    0 |                        0 |
|         hanoi4 |                       0 |                        0 |
|         hanoi8 |                       2 |                        0 |
|        hanoi12 |                      42 |                        1 |
|        hanoi16 |                     813 |                       31 |
|        hanoi20 |                   14807 |                      604 |
|         intnat |                      16 |                        1 |
|          merge |                       0 |                        0 |
|    mergesort10 |                       0 |                        0 |
|   mergesort100 |                       - |                        - |
|  mergesort1000 |                       - |                        - |
|  missionaries2 |                       2 |                        0 |
|  missionaries3 |                       9 |                        0 |
|            mma |                   36246 |                     2013 |
|           mul8 |                    1074 |                       32 |
|          mul16 |                    6704 |                      205 |
|          mul32 |                   22961 |                      752 |
|        natlist |                      33 |                        0 |
|       langton6 |                       - |                    35152 |
|       langton7 |                       - |                    52209 |
|         logic3 |                       0 |                        0 |
|        oddeven |                    1372 |                     3639 |
|  permutations6 |                      36 |                        2 |
|  permutations7 |                       ? |                       97 |
|          omul8 |                   58671 |                       38 |             
|         omul32 |                   22992 |                      672 |
|          order |                       0 |                        0 |
|    quicksort10 |                      16 |                        0 |
|   quicksort100 |                       - |                        - |
|  quicksort1000 |                       - |                        - |
| searchinconditions |                   0 |                        0 |
| soundnessofparallelengines |           0 |                        0 |
|      revnat100 |                       2 |                        0 |
|     revnat1000 |                     240 |                       47 |
|    revnat10000 |                   25160 |                     6193 |
|        sieve20 |                       0 |                        0 |
|       sieve100 |                      25 |                        0 |
|      sieve1000 |                    5121 |                       79 |
|      sieve2000 |                   30920 |                      444 |
|     sieve10000 |                       - |                     6193 |
|          tak18 |                     150 |                        6 |
|          tak36 |                   12578 |                      438 |
|  tautologyhard |                       0 |                        0 |

 [1] Durán, F., Garavel, H. (2019). The Rewrite Engines Competitions: A RECtrospective. In: Beyer, D., Huisman, M., Kordon, F., Steffen, B. (eds) Tools and Algorithms for the Construction and Analysis of Systems. TACAS 2019. Lecture Notes in Computer Science(), vol 11429. Springer, Cham. https://doi.org/10.1007/978-3-030-17502-3_6
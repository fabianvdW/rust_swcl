# rust_swcl
Client, der beim Software-Challenge Finale für das Lise-Meitner-Gymnasium G8GTS spielt

## Move Generation
Basiert auf Bitboards. Siehe https://www.chessprogramming.org/Bitboards

## Suche
Principal-Variation-Search in einem Negamax-Framework. Siehe https://www.chessprogramming.org/Principal_Variation_Search

Moveordering mit relative history heuristic.  Siehe https://www.chessprogramming.org/Relative_History_Heuristic

Caching durch Zobrist-Hashing und simpler Replacement-Heuristik. https://www.chessprogramming.org/Transposition_Table

Quiesence-Search wurde einige Zeit ausprobiert, aber verworfen.

Genauso einige Pruning-Techniken wie Futility-Pruning & Razoring. Das diese aber unerfolgreich waren lässt sich aber durch den krassen Odd-Even-Effekt der Boardbewertungsfunktion erklären.

Alle Tests wurden im lokalen Spielleiter (siehe Java-Client) ausgiebig getestet.
## Boardbewertung
Zusammengebaut und jede Änderung getestet. Etwas mehr Wissen über das Spiel und Interesse an dem Spiel wäre hier wahrscheinlich von Vorteil gewesen.

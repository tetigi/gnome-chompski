# conversate

Turns out that ChatGPT is actually pretty useful. 

Limited customisation at the moment. Prefix input with `!` to just ask the teacher and not pollute the conversation state.

```
❯ ./target/debug/conversate
>  Dzień dobry!
Teacher: Good morning!
---
Partner: Dzień dobry! Witam serdecznie. Jak się masz?

>  Wszystko dobrze, dziękuję. A u ciebie? Co zrobisz dzisiaj?
Teacher: "All good, thank you. And how about you? What are you going to do today?"
---
Partner: Też wszystko w porządku, dziękuję. Dzisiaj planuję pójść na spacer i poćwiczyć jogę. A ty, masz jakieś plany na dzisiaj?

>  Niestety nie mam plan dzisiaj
Teacher: "Unfortunately, I don't have any plans today."

The correct Polish sentence would be "Niestety nie mam żadnego planu na dziś."
---
Partner: Może chciałbyś obejrzeć jakiś film lub poczytać książkę? Może to być dobry czas na naukę nowych słówek w języku polskim lub na odkrycie czegoś nowego o Polsce.

>  ! what does 'obejrzeć' mean?
Teacher: "Obejrzeć" means "to watch" or "to view" in English.

>  ! is 'obejrzę' the correct form for 'I watch'?
Teacher: Yes, "obejrzę" is the correct form for "I will watch".

>  Tak, chciałbym obejrzeć film! W kinie? Może coś eksytuacego
Teacher: Tak, chciałbym obejrzeć film! W kinie? Może coś ekscytującego? (Corrected spelling of 'ekscytującego')
---
Partner: W kinie jest teraz wiele dobrych filmów, szczególnie w gatunku akcji i sci-fi. Może zainteresowałby Cię film "Tenet" Christophera Nolana - to nowa produkcja, która wciągnie Cię w świat podróży w czasie i walki z mi
ędzynarodową organizacją przestępczą. Co o tym myślisz?

>  ! what does ekscytujący mean?
Teacher: "Ekscytujący" means "exciting" in English.

```

## Installation

Requires a `.env` file with an `API_TOKEN` field set to your [api token](https://platform.openai.com/account/api-keys).

Then just do:

`cargo run`


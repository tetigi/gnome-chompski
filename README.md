## Gnome Chompski

_Turns out that ChatGPT is actually pretty useful._

Gnome Chompski is a ChatGPT-powered language gnome that you can message over Discord. He will have language-based discussions with you whenever you want, except sometimes he's wrong because he's a Gnome, not a linguist.

### Yeah, but what is it _actually_?

Gnome Chompski is a Discord bot that interacts with ChatGPT in an opinionated way to provide an educational environment for learning a language (currently just Polish).

![discord example](/resources/ex_discord.png)

## Installation

### Get yo'self an OpenAI API token

### Get yo'self an Discord API token

### Start the bot

Create a `.env` file with the above tokens:

```
DISCORD_API_TOKEN=...
OPENAI_API_TOKEN=...
```

Then start up Gnome Chompski!

```
cargo run
```

## Commands

Gnome Chompski understands all kinds of useful commands. The place to start is with a `chat!` invocation:

```
> !chat <topic/subject/a question>
```

Gnome Chompski will now strike up a conversation with you on the topic of your choice. You can reply to these messages, and he will reply back to you.

Gnome Chompski will also correct any mistakes you make when you write to him. So helpful!

At any point in a conversation, you can send the following special commands:

- `!ask <question>` -> Ask Gnome Chompski a question without interrupting the conversation flow.
- `!ex <word>` -> Gnome Chompski will provide you 3 example sentences containing that word, with translations.
- `!cases <word>` -> Gnome Chompski will enumerate the different cases of the provided word.
- `!def <word>` -> Gnome Chompski will define what `word` means.
- `!chat <topic>` -> Gnome Chompski will start a new conversation as if nothing had ever happened!

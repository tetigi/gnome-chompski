## Gnome Chompski

_Turns out that ChatGPT is actually pretty useful._

Gnome Chompski is a ChatGPT-powered language gnome that you can message over Discord. He will have language-based discussions with you whenever you want, except sometimes he's wrong because he's a Gnome, not a linguist.

### Yeah, but what is it _actually_?

Gnome Chompski is a Discord bot that interacts with ChatGPT in an opinionated way to provide an educational environment for learning a language (currently just Polish).

![discord example](/resources/ex_discord.png)

## Installation

### Get yo'self an OpenAI API token

Simply follow the instructions on the [OpenAI website](https://platform.openai.com/account/api-keys).

### Get yo'self an Discord API token

Follow the [Getting Started](https://discord.com/developers/docs/getting-started) guide on Discord to get set up.

You simply need to follow it to the [Installing your app](https://discord.com/developers/docs/getting-started#installing-your-app) section, at which point you will get an 'installation URL'. You can paste this into your browser to add Gnome Chompski (or whatever you name the bot) to your groups.

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

### Chatting with the bot

Gnome Chompski only chats with people 1:1 - he will cowardly refuse to talk in a non-private channel.

## Commands

Gnome Chompski understands all kinds of useful commands. The place to start is with a `chat!` invocation:

```
> !chat <topic/subject/a question>
```

Gnome Chompski will now strike up a conversation with you on the topic of your choice. You can reply to these messages, and he will reply back to you.

Gnome Chompski will also correct any mistakes you make when you write to him. So helpful!

At any point in a conversation, you can send the following special commands:

- `!chat <topic>` -> Gnome Chompski will start a new conversation as if nothing had ever happened!
- `!ask <question>` -> Ask Gnome Chompski a question without interrupting the conversation flow.
- `!ex <word>` -> Gnome Chompski will provide you 3 example sentences containing that word, with translations.
- `!cases <word>` -> Gnome Chompski will enumerate the different cases of the provided word.
- `!def <word>` -> Gnome Chompski will define what `word` means.
- `!undo` -> Forget the last message and reply (useful if you want to retry a sentence).
- `!help` -> Print a helpful help message.

## (Optional) Authentication

If you would like to host Gnome Chompski on behalf of others, Gnome Chompski creates user-sessions for each user on startup. These are (currently) temporary, so are thrown away on restart.

### Auth Strategies

If you start up Gnome Chompski with no arguments, he will start in `NO_AUTH` mode - that is, anyone who connects to him will begin a new session!

Alternatively, you can provide Gnome Chompski with a `tokens-file` at startup (a file containing a new-line delimited list of tokens you'd like to support) - he will then load these into a local database, and require that new users join by providing one of these tokens. Once a user is authenticated once, they will no longer be prompted (unless you delete the database).

You can add new tokens to this file whenever you want and they will be added to the database. Removing tokens from the file will _not_ remove them from the database.

## Next Steps

- Get Gnome Chompski set up as a general chat-bot
- Add better support for other languages
- ???
- Profit

### Wishlist

- ~Ability to handle multiple connections from multiple users at the same time~
- ~Ability to limit users based on a token provided at startup~
- ~Add 'long request' checks or messages when something goes wrong.~
- Add tests for token stuff
- Ability to provide cloud-hosting for non-technical users

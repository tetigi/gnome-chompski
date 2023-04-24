# Discord-hosted learning bot

**Goal**: Chat with teach-bot through Discord, so that I can easily spin up a conversation on the go.

**Stretch goal**: Make it easy for others to do the same if they wish.

### Example Workflow

```
> !chat "We're in a restaurant. I am a customer, and you are a waiter"

... chat begins ...

~ A co do picia?
> nie wiem może woda

~ Tak, bla bla
~ (That was correct, except that 'woda' is biernik so should be 'wodę')

> !cases woda

~ woda / wody / wodę ...

... etc. ...
```

### What we have so far:

- A very basic chat setup with prompt setup and then the teacher / conversation flow
- A 'ping' bot using the 'serenity' library, which responds to commands.

### The concrete flow we need:

- Conversations are stored in memory. Don't need anything special there.
- Conversations are stored per user.
- Prompts:
    - !chat <topic/subject/prompt> - starts a new chat from fresh. Clears old state.
    - !ask <question> - ask the teacher a question
    - !cases <word> - gets all the cases for a particular word
    - !ex <word> - get example sentences of usage of a particular word

Should be easy enough.

### Next steps:

1. ~Tidy up code to get ready for proper integration~
2. Add handlers for the commands above
3. ... that's it? Should be pretty simple

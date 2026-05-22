![](banner.png)


## An alarm clock that can see you

[TBD]



## History

### 15-year-old Attempt


Jokes aside, this is one of my oldest ideas as an engineer. When I was 15, I tried a very similar approach using a *very* sketchy force transducer made from my parent's bathroom scale. It was SO bad, but I was 15 so idrc. This is what it looked like :sob::

<img width="972" height="513" alt="image" src="https://github.com/user-attachments/assets/8fc25f43-07f6-474b-961e-92c9a2626700" />



This technically worked for a couple days, but I had to plug my laptop into it every night and I gave up on that within a week. I remember at the time contemplating running a CNN on a Raspberry PI that could monitor a video feed of my bed. I actually found the blog post where I was thinking about this:

> "I also considered the option of training an AI to use a picture to determine if I was in bed or not. I also ended up deciding against this because of the sheer complexity."

From a [blog](https://rannsyt.blogspot.com/2020/10/i-built-smart-bed-blog.html) in 2020.

And it's good that I dropped that idea, because a conventional CNN would be so sensitive to the environment--if I could even get it trained in the first place. It would require a lot of training data, and the moment my room changed (new picture, different bedding, etc), the model could easily start spitting out garbage.

### The GPT

Generative **Pretrained** Transformers (GPTs) makes this task not just possible, but extremely simple. You can just take a VLM with a text query of "Is someone in this bed?" and it will work almost perfectly in any setting, so long that there's a good enough camera angle. The primary issue is security: we don't want to call any API for our model because:

1. that would cost approximately one trillion dollars
2. nobody wants to livestream themself sleeping

So, to address this, we can run the model on any relatively powerful chip (I chose the 8gb Jetson Orin Nano). The model is run with [`llama.cpp`](https://github.com/ggml-org/llama.cpp) (see [`src/llama.rs`](/src/llama.rs)), and I'm using an absolutley tiny 450M VLM ([`LiquidAI/LFM2.5-VL-450M-GGUF`](https://huggingface.co/LiquidAI/LFM2.5-VL-450M-GGUF)) which is shockingly reliable.

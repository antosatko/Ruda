from manim import *



# Hello world scene
# Hvae "hello world" go smoothly to the center of the screen and then fade out
class Test(Scene):
    def construct(self):
        # Create text
        hello = MarkupText("Hello <b>World</b>!", color=BLUE, stroke_width=0)
        # smooth animation
        self.play(Write(hello))
        self.wait(1)
        self.play(hello.animate.shift(UP*2))
        # Play animation
        self.play(FadeOut(hello))
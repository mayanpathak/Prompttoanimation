from manim import *
import numpy as np

class GeneratedScene(Scene):
    def construct(self):
        circle = Circle(radius=0.7, color=RED, fill_opacity=0.8)
        circle.move_to(LEFT * 6)

        self.play(FadeIn(circle), run_time=1)
        self.play(circle.animate.shift(RIGHT * 12), run_time=3.5, rate_func=linear)
        self.play(FadeOut(circle), run_time=1)
        self.wait(0.5)


        
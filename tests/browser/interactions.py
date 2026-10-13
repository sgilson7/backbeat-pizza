"""Backbeat Pizza's browser interactions, run by check.py in every engine.

check.py has already loaded the page, waited for it to report ready, and let
it idle. This file plays as a person would: it opens level 1, builds a pizza
with the buttons, serves it, and then plays the level to the end, asking the
core's solver for each pizza (through window.backbeat, the same door the
buttons use) and pressing the real serve button.

Expected values are worked out by hand from data/: level 1 is "Soundcheck",
one drummer, patience 5, three toppings, and the first topping is Pepperoni.
Every wait is on a selector or a data-* attribute, never a fixed time.
"""

import json
import pathlib

DATA = pathlib.Path(__file__).resolve().parents[2] / "data"
COPY = json.loads((DATA / "copy.json").read_text())


def run(page, check, engine):
    check(page.get_attribute("body", "data-mode") == "title", "the title screen is shown first")
    check(page.inner_text("#title h1") == COPY["title"], "the title comes from data/copy.json")
    check(page.locator("#level-list button").count() == 10, "ten levels are listed")

    page.click("#level-list button >> nth=0")
    page.wait_for_selector("body[data-mode='intro']")
    check(page.inner_text("#intro-name") == "Soundcheck", "level 1 is Soundcheck")
    rules = page.inner_text("#intro-rules")
    check("1 drummer to feed." in rules and "5 pizzas" in rules, f"level 1's rules name its drummer and patience: {rules!r}")

    page.click("#start")
    page.wait_for_selector("body[data-mode='play']")
    check(page.inner_text("#patience") == "5 of 5", "patience starts full")
    check(page.locator("#toppings button").count() == 3, "level 1 offers three toppings")

    page.click("#toppings button >> nth=0")
    page.wait_for_selector("#toppings button[aria-pressed='true']")
    check("Pepperoni" in page.inner_text("#pizza"), "the pressed topping is on the pizza")

    page.click("#serve-buttons button >> nth=0")
    page.wait_for_selector("#serve-buttons button[disabled]")
    check(page.inner_text("#patience") == "4 of 5", "a serve uses one patience")
    page.wait_for_selector("#log .entry", timeout=20000)
    check(page.locator(".bubble .verdict").count() == 1, "the drummer's hint shows in a bubble with its verdict")

    served = 1
    while page.get_attribute("body", "data-mode") == "play" and served < 6:
        fed = page.evaluate("() => window.backbeat.ui().play.drummers[0].fed")
        if fed:
            break
        page.wait_for_selector("#serve-buttons button:not([disabled])", timeout=20000)
        page.evaluate("() => window.backbeat.act('solver.suggest', '0')")
        page.click("#serve-buttons button >> nth=0")
        served += 1
        page.wait_for_function(f"() => document.querySelectorAll('#log .entry').length >= {served}", timeout=20000)
    page.wait_for_selector("body[data-mode='won']", timeout=20000)
    check(True, f"the level is cleared from the hints alone in {served} pizzas")
    check(page.inner_text("#won h1") == COPY["won_heading"], "the level-cleared card is shown")

    page.click("#next")
    page.wait_for_selector("body[data-mode='intro']")
    check(page.inner_text("#intro-name") == "Garage Gig", "next goes to level 2")

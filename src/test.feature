
@feature-tag
Feature: This is a feature file


# Surprise comment
Background:
  Given I am doing a test
    When I test
  Then I succeed
     """
     I can have docstrings
     """
  And a thing
  But another
    | first      | second           | third          |
    | a thingo   | another thingo   | final thingo   |
    | a thingo 2 | another thingo 2 | final thingo 2 |
  And then it was fun

@tag1 @tag2 @tag-life_woo
Scenario: A second scenario test
  Given I have not been testing much
  Then I should probably start doing it


  Scenario: I am lightly tabbed
    Given I am lightly tabbed
    Then handle how tabbed I am

@taglife
Scenario Outline: eating
  Given there are <start> cucumbers
  When I eat <eat> cucumbers
  Then I should have <left> cucumbers

  @another-misfeature-of-cucumber
  Examples:
    | start | eat | left |
    |    12 |   5 |    7 |
    |    20 |   5 |   15 |

Scenario: A step with a doc comment and no new line at the end of the doc
Given a
"""
there's no newline following this docstring
"""
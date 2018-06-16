

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
    | first | second | third |
    | a thingo | another thingo | final thingo |
    | a thingo 2 | another thingo 2 | final thingo 2 |
  And then it was fun


Scenario: A second scenario test
  Given I have not been testing much
  Then I should probably start doing it


  Scenario: I am lightly tabbed
    Given I am lightly tabbed
    Then handle how tabbed I am

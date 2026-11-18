@gherkin43 @step_arguments
Feature: Portable step arguments
  Authors can share a step containing a table and a doc string with Gherkin 43
  consumers without changing the argument order or losing either value.

  Scenario Outline: Preserve both arguments in either order for every step keyword
    Given a feature whose "<keyword>" step has "<arguments>" arguments
    When the feature is parsed
    Then parsing succeeds with both argument values on the same step
    And the following step remains separate

    Examples:
      | keyword | arguments    |
      | Given   | table-doc    |
      | When    | table-doc    |
      | Then    | table-doc    |
      | And     | table-doc    |
      | But     | table-doc    |
      | *       | table-doc    |
      | Given   | doc-table    |
      | When    | doc-table    |
      | Then    | doc-table    |
      | And     | doc-table    |
      | But     | doc-table    |
      | *       | doc-table    |

  Scenario Outline: Preserve existing features with fewer arguments
    Given a feature whose "Given" step has "<arguments>" arguments
    When the feature is parsed
    Then parsing succeeds with "<arguments>" arguments
    And the following step remains separate

    Examples:
      | arguments |
      | none      |
      | doc       |
      | table     |

  Scenario Outline: Reject repeated argument kinds on one step
    Given a feature whose "Given" step has "<arguments>" arguments
    When the feature is parsed
    Then parsing fails

    Examples:
      | arguments     |
      | doc-doc       |
      | doc-table-doc |
      | table-doc-table |

  Scenario Outline: Preserve literal and recognized table escapes
    Given a feature with the "<encoding>" table cell encoding
    When the feature is parsed
    Then the "<encoding>" table cell value is preserved exactly
    And the following step remains separate

    Examples:
      | encoding          |
      | unknown escape    |
      | trailing slash    |
      | escaped pipe      |
      | escaped slash     |
      | escaped newline   |
      | mixed escapes     |

  Scenario Outline: Authors use the current official dialect vocabulary explicitly
    Given the published Gherkin 43 vocabulary cases for "<language>"
    When those features are parsed with the Gherkin 43 catalogue
    Then every published keyword retains its intended structure and text

    Examples:
      | language |
      | af |
      | amh |
      | be |
      | da |
      | en-old |
      | en-tx |
      | eo |
      | fa |
      | ga |
      | gj |
      | it |
      | ja |
      | ka |
      | ko |
      | ml |
      | ne |
      | nl |
      | ru |
      | sr-Cyrl |
      | ta |
      | tr |
      | uz |
      | vi |
      | zh-CN |

  Scenario: Existing consumers retain an obsolete Persian feature alias
    Given a feature using the legacy Persian feature alias
    When it is parsed with legacy and Gherkin 43 catalogue policies
    Then legacy default and explicit parsing preserve the feature
    And the Gherkin 43 policy rejects the obsolete feature alias

  Scenario: New dialect availability is explicit
    Given the new Amharic Belarusian and Malayalam dialects
    When dialect availability is requested for both catalogue policies
    Then only the Gherkin 43 policy reports those new dialects

  Scenario Outline: Explicit official doc-string content matches the published oracle
    Given the official Gherkin 43 doc-string fixture "<fixture>"
    When it is parsed with official doc-string content semantics
    Then every native doc-string value equals the external official AST content

    Examples:
      | fixture                                  |
      | docstrings.feature                       |
      | docstrings.crlf.feature                  |
      | step_with_datatable_and_docstring.feature |

  Scenario Outline: Existing constructors retain legacy doc-string values
    Given a feature containing a legacy doc-string payload
    When it is parsed using the "<policy>" legacy-content constructor
    Then the original delimiter-adjacent newlines remain unchanged

    Examples:
      | policy          |
      | default         |
      | explicit legacy |
      | English         |
      | latest dialect  |

  Scenario Outline: Official doc-string mode rejects an unclosed block
    Given a feature with a "<closure>" doc-string closure
    When it is parsed with official doc-string content semantics
    Then parsing fails

    Examples:
      | closure     |
      | missing     |
      | mismatched  |
      | same line   |

  @phase15
  Scenario Outline: Strict table rows preserve a rectangular contract
    Given a "<kind>" table with a "<width>" later row
    When the table feature uses "Gherkin 43" semantics
    Then the table parser reports inconsistent cell count

    Examples:
      | kind     | width   |
      | step     | shorter |
      | step     | longer  |
      | Examples | shorter |
      | Examples | longer  |

  @phase15
  Scenario Outline: Rectangular tables preserve all values under strict semantics
    Given a "<kind>" table with a "matching" later row
    When the table feature uses "Gherkin 43" semantics
    Then the "<kind>" table preserves both columns

    Examples:
      | kind     |
      | step     |
      | Examples |

  @phase15
  Scenario Outline: Existing constructors preserve excess-cell truncation
    Given a "<kind>" table with a "longer" later row
    When the table feature uses "<policy>" semantics
    Then the "<kind>" table preserves both columns

    Examples:
      | kind     | policy          |
      | step     | default         |
      | Examples | default         |
      | step     | explicit legacy |
      | Examples | explicit legacy |
      | step     | English         |
      | Examples | latest dialect  |

  @phase15
  Scenario: Strict parsing rejects the published invalid table fixture
    Given the official Gherkin 43 inconsistent-cell-count fixture
    When the table feature uses "Gherkin 43" semantics
    Then the table parser reports inconsistent cell count

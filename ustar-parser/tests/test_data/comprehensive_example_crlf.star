# Comprehensive STAR file demonstrating all major grammar components
# This file showcases every major construct defined in the ustar grammar

# ===== DATA BLOCK WITH VARIOUS DATA TYPES =====
data_comprehensive_example

# Simple data items with different value types
_simple_text_value      hello_world
_numeric_value         42.5
_boolean_value         true
_quoted_underscore     '_another_name'

# Single quoted strings (can contain double quotes and spaces)
_single_quoted         'Hello "world" with spaces'
_single_quote_escapes  'Don''t forget the apostrophe''s'
_single_quote_complex  'Mix of "quotes" and ''escapes'''

# Double quoted strings (can contain single quotes and spaces)  
_double_quoted         "Hello 'world' with spaces"
_double_quote_escapes  "She said ""Hello""to me"
_double_quote_complex  "Mix of 'quotes' and ""escapes"""

# Frame codes (start with $)
_frame_code_simple     $frame1
_frame_code_complex    $my_complex_frame_123

# Semicolon-bounded text string (multiline with embedded semicolons)
_multiline_text
;
This is a multiline text string.
It can contain semicolons; like this one.
It can span multiple lines.
Even blank lines are preserved.

Special characters: !@#$%^&*()
;

# ===== DATA LOOP (Simple) =====
loop_
    _atom_site_label
    _atom_site_type_symbol
    _atom_site_fract_x
    _atom_site_fract_y
    _atom_site_fract_z
    C1  C  0.1234  0.5678  0.9012
    N1  N  0.2345  0.6789  0.0123
    O1  O  0.3456  0.7890  0.1234
    H1  H  0.4567  0.8901  0.2345
stop_

# ===== DATA LOOP (Nested) =====
loop_
    _struct_conf_type_id
    loop_
        _struct_conf_atom_site_label
        _struct_conf_atom_site_auth_seq_id
    stop_
    HELX_P1
        CA  123
        CB  124  
        CG  125
    stop_
    STRN_S1
        CA  456
        CB  457
    stop_
stop_

# ===== SAVE FRAMES =====
save_frame_example_1

    _save_frame_category     molecular_structure
    _description            'This is a save frame example'
    _created_by             ustar_parser
    
    # Data within save frame
    _temperature            298.15
    _pressure              'atmospheric'
    
    # Loop within save frame
    loop_
        _bond_atom_1
        _bond_atom_2
        _bond_length
        C1  C2  1.54
        C2  C3  1.52
        N1  C1  1.47
    stop_

save_

save_frame_example_2

    _another_category       experimental_data
    _methodology           "X-ray crystallography"
    
    # Semicolon text in save frame
    _experimental_details
;
The crystal structure was determined using
X-ray crystallography at 100K.

Data collection parameters:
- Wavelength: 0.71073 Angstrom
- Temperature: 100(2) K  
- Crystal system: Monoclinic
;

save_

# ===== GLOBAL BLOCK =====
global_

# Global data items
_global_version          2.1
_global_format          'STAR'
_global_software        'ustar parser'

# Global loop
loop_
    _software_name
    _software_version
    _software_author
    ustar               "1.0"        "Gary Thompson"
    crystallography     "2024.1"     "Various Authors"
stop_

# ===== ANOTHER DATA BLOCK =====
data_second_example

# Demonstrate edge cases and special characters
_special_chars          '!@#$%^&*()_+-=[]{}|;:,.<>?'
_ascii_only            'standard_ascii_text'
_mixed_case_VALUE       MixedCaseValue
_hyphenated_value       some-hyphenated-text
_dotted_value          some.dotted.value

# Keywords that must be quoted to be used as values
_keyword_as_value       'data_'
_another_keyword       'loop_'
_save_keyword          'save_'
_stop_keyword          'stop_'
_global_keyword        'global_'

# Numbers and scientific notation
_integer               123
_float                 123.456
_scientific            1.23e-4
_negative              -456.789

# Frame code references
_reference_frame       $reference_structure
_molecular_frame       $molecule_001

# Empty and minimal values
_minimal_value         .
_question_mark         ?
_single_char           x

# Complex nested quotes
_complex_quotes        'She said "He replied ''Yes ''to the question"'
_more_complex         "It's a ""complex ""situation with 'mixed 'quotes"

# Final semicolon-bounded text with complex content
_final_multiline
;
This is the final demonstration of semicolon-bounded text.

It can contain:
- Multiple paragraphs
- Special characters: !@#$%^&*()
- Quotation marks: 'single' and "double"
- Even semicolons in the middle; like this
- Frame references: $frame_ref
- Underscores: _like_this
- And keywords: data_ loop_ save_ stop_ global_

The text continues until a line starting with semicolon.
;

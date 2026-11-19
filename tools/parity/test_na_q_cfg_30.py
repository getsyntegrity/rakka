"""Tests of the NA:Q-CFG-30 signature: the cases that enter and near misses that must not."""
import unittest

import na_q_cfg_30 as q


def outputs(oracle_number, rakka_number, before="NUM-SHORT x -> ok ", after=""):
    return (before + oracle_number + after).encode(), (before + rakka_number + after).encode()


class ClassifierAccepts(unittest.TestCase):
    def test_the_five_doubles_of_the_sample_where_jdk_17_is_wrong(self):
        for oracle, rakka in [
            ("-4.8726281932648956E19", "-4.872628193264896E19"),
            ("9.841100260380799E18", "9.8411002603808E18"),
            ("2.3851354565708683E25", "2.3851354565708684E25"),
            ("9.231177483585921E18", "9.23117748358592E18"),
            ("-3.5624613975366162E25", "-3.5624613975366163E25"),
        ]:
            self.assertTrue(q.is_q_cfg_30(*outputs(oracle, rakka)), oracle)

    def test_a_longer_oracle_string_that_is_not_one_digit_longer(self):
        # 2e23: JDK 17 prints 17 digits, the shortest is 2 digits (the one digit 2 is shown as 2.0)
        self.assertTrue(q.is_q_cfg_30(*outputs("1.9999999999999998E23", "2.0E23")))

    def test_an_exact_tie_that_the_oracle_resolves_to_the_odd_digit(self):
        self.assertTrue(q.is_q_cfg_30(*outputs("5.804742410468123E14", "5.804742410468122E14")))

    def test_the_number_inside_a_larger_document(self):
        before, after = '{"a":1,"b":"text 1.5","c":', ',"d":[1.0E7]}'
        self.assertTrue(q.is_q_cfg_30(*outputs("9.841100260380799E18", "9.8411002603808E18", before, after)))


class ClassifierRejectsNearMisses(unittest.TestCase):
    def test_identical_outputs(self):
        self.assertFalse(q.is_q_cfg_30(*outputs("9.8411002603808E18", "9.8411002603808E18")))

    def test_a_different_double_one_ulp_away(self):  # condition a
        # 9.841100260380801E18 is the next double: it is rakka's correct text for another value
        self.assertFalse(q.is_q_cfg_30(*outputs("9.841100260380799E18", "9.841100260380801E18")))

    def test_a_tie_that_rakka_resolves_to_the_odd_digit(self):  # condition b
        self.assertFalse(q.is_q_cfg_30(*outputs("5.804742410468122E14", "5.804742410468123E14")))

    def test_rakka_with_more_digits_than_needed(self):  # condition b
        self.assertFalse(q.is_q_cfg_30(*outputs("9.8411002603808E18", "9.841100260380799E18")))

    def test_neither_text_is_the_jdk_19_one(self):  # condition b
        # both identify the same double, but rakka's text is not the shortest closest one either
        self.assertFalse(q.is_q_cfg_30(*outputs("9.841100260380799E18", "9.841100260380798E18")))

    def test_two_digits_of_difference_that_are_not_the_jdk_bug(self):  # condition b
        # both strings identify the double but neither is the shortest closest one
        self.assertFalse(q.is_q_cfg_30(*outputs("9.84110026038079E18", "9.841100260380799E18")))

    def test_the_oracle_is_already_the_jdk_19_text(self):  # condition c
        self.assertFalse(q.is_q_cfg_30(*outputs("2.0E23", "2.0E23", after=" ")))
        self.assertFalse(q.is_q_cfg_30(b"x 2.0E23 y", b"x 2.0E23 z"))

    def test_the_oracle_text_is_the_jdk_19_digits_with_a_padding_zero(self):  # condition c
        self.assertFalse(q.is_q_cfg_30(*outputs("2.00E23", "2.0E23")))
        self.assertFalse(q.is_q_cfg_30(*outputs("100.00", "100.0")))

    def test_a_different_notation(self):  # condition d
        self.assertFalse(q.is_q_cfg_30(*outputs("9841100260380799000.0", "9.8411002603808E18")))

    def test_a_different_exponent(self):  # condition d
        # same double (the one nearest to 1e23); the digits sit on different powers of ten
        self.assertFalse(q.is_q_cfg_30(*outputs("9.999999999999999E22", "1.0E23")))

    def test_a_difference_in_another_part_of_the_document(self):  # condition e
        oracle = b'{"a":1,"b":9.841100260380799E18}'
        self.assertFalse(q.is_q_cfg_30(oracle, b'{"a":2,"b":9.8411002603808E18}'))
        self.assertFalse(q.is_q_cfg_30(oracle, b'{"a":1,"b":9.8411002603808E18} '))

    def test_a_second_number_that_differs(self):  # condition e
        oracle = b"9.841100260380799E18 2.3851354565708683E25"
        self.assertFalse(q.is_q_cfg_30(oracle, b"9.8411002603808E18 2.3851354565708684E25"))

    def test_text_that_is_not_utf8(self):
        self.assertFalse(q.is_q_cfg_30(b"\xff9.841100260380799E18", b"\xff9.8411002603808E18"))


class ExactArithmetic(unittest.TestCase):
    def test_the_nearest_double_of_the_extremes(self):
        self.assertEqual(q.Literal("4.9E-324").double(), (False, 1, -1074))
        self.assertEqual(q.Literal("-0.0").double(), (True, 0, 0))
        self.assertEqual(q.Literal("1.7976931348623157E308").double(), (False, 2**53 - 1, 971))
        self.assertIsNone(q.Literal("1.7976931348623159E308").double())

    def test_shortest_digits_follow_the_javadoc(self):
        # one digit is shown as two (4.9E-324, not 5.0E-324); an exact tie takes the even digit
        self.assertEqual(q.shortest_digits(1, -1074), ("49", -324))
        self.assertEqual(q.shortest_digits(*q.Literal("5.804742410468123E14").double()[1:]), ("5804742410468122", 14))
        self.assertEqual(q.shortest_digits(*q.Literal("1.0E-7").double()[1:]), ("1", -7))


if __name__ == "__main__":
    unittest.main()

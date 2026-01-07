#!/usr/bin/env python3
"""
Simple text sender for testing Kokoro TTS in Dora dataflow.
Sends test text once and then stops.
"""

import pyarrow as pa
from dora import Node


def main():
    """Main function to send test text."""

    # Test text - English by default
    english_text = """Artificial intelligence is revolutionizing the way we interact with technology. From voice assistants to autonomous vehicles, AI systems are becoming increasingly sophisticated. Machine learning algorithms can now process vast amounts of data, recognize patterns, and make decisions with remarkable accuracy."""

    chinese_text = """我们说中国式现代化是百年大战略，这又分为三个阶段。第一个阶段，我们先用30年时间建成了独立完整的工业体系和国民经济体系；再用40年，到2021年，全面建成了小康社会。我们现在正处于第三个阶段，这又被分成上下两篇：上半篇是到2035年基本实现社会主义现代化；下半篇是到本世纪中叶，也就是2050年，建成社会主义现代化强国。"""

    # Use English text by default (can be changed based on LANGUAGE env var)
    import os
    language = os.getenv('LANGUAGE', 'en')

    if language in ['zh', 'chinese', 'ch']:
        test_text = chinese_text
    else:
        test_text = english_text

    print(f"[TextSender] Using language: {language}")
    print(f"[TextSender] Text length: {len(test_text)} characters")
    print(f"[TextSender] Text: {test_text[:100]}...")

    node = Node()

    # Send text once
    print("[TextSender] Sending text to TTS...")
    node.send_output("text", pa.array([test_text]))
    print("[TextSender] Text sent successfully")

    # Wait for a bit to allow TTS to process
    import time
    time.sleep(2)

    # Keep node alive briefly to ensure message is delivered
    for event in node:
        if event["type"] == "STOP":
            break

    print("[TextSender] Text sender stopped")


if __name__ == "__main__":
    main()

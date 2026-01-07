#!/usr/bin/env python3
"""
Static text sender node that waits before sending text.
"""

import time
import pyarrow as pa
from dora import Node

def main():
    """Static node that sends text after a delay."""
    
    print("=" * 80)
    print("Static Text Sender - Waiting then sending text")
    print("=" * 80)
    
    node = Node()
    
    # Mixed language text for leakage testing
    mixed_text = """Hello world. 你好世界. This is a test. 这是一个测试. Welcome to Beijing. 欢迎来到北京. AI technology. 人工智能技术. Thank you very much. 非常感谢."""
    
    print(f"\nText to send ({len(mixed_text)} characters)")
    print("-" * 60)
    print("Mixed content for leakage testing:")
    print(mixed_text)
    print("-" * 60)
    
    # Wait for system to be ready
    print("Waiting 5 seconds for all nodes to initialize...")
    time.sleep(5)
    
    # Send the text
    start_time = time.time()
    print(f"\nSending text at {time.strftime('%H:%M:%S')}...")
    
    node.send_output(
        "text_output",
        pa.array([mixed_text]),
        metadata={
            "session_id": "leakage_test",
            "char_count": len(mixed_text),
            "start_time": start_time,
            "test_type": "mixed_language_leakage"
        }
    )
    
    print(f"✓ Text sent to text-segmenter")
    
    # Keep running to observe results
    print("\nWaiting for TTS processing...")
    
    # Wait for events or timeout
    timeout = 120  # 2 minutes
    start = time.time()
    
    for event in node:
        if event["type"] == "STOP":
            break
        if time.time() - start > timeout:
            print("Timeout reached, exiting")
            break
    
    print("\n" + "=" * 80)
    print("Text sender completed")
    print("=" * 80)


if __name__ == "__main__":
    main()
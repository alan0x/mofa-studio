#!/usr/bin/env python3
"""
Convert FunASR PyTorch models to ONNX format for improved inference performance.

This script converts FunASR models from PyTorch (.pt) to ONNX format, which can
provide better inference performance and compatibility across different platforms.
"""

import os
import sys
import argparse
from pathlib import Path
from typing import Optional, Dict, List
import json

# Progress bar imports
try:
    from tqdm import tqdm
except ImportError:
    print("Installing tqdm for progress bars...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "tqdm"])
    from tqdm import tqdm


def check_funasr_installation():
    """Check if FunASR is installed."""
    try:
        import funasr
        return True
    except ImportError:
        print("FunASR not installed. Installing...")
        try:
            import subprocess
            subprocess.check_call([sys.executable, "-m", "pip", "install", "funasr"])
            return True
        except Exception as e:
            print(f"Failed to install FunASR: {e}")
            print("Please install manually: pip install funasr")
            return False


def convert_paraformer_to_onnx(model_dir: Path, output_dir: Optional[Path] = None):
    """Convert Paraformer ASR model to ONNX format.
    
    Args:
        model_dir: Directory containing the PyTorch model
        output_dir: Output directory for ONNX model (default: same as model_dir)
    """
    print(f"\n📦 Converting Paraformer model to ONNX")
    print(f"   Source: {model_dir}")
    
    if output_dir is None:
        output_dir = model_dir
    
    # Check if model.pt exists
    model_pt = model_dir / "model.pt"
    if not model_pt.exists():
        print(f"   ❌ model.pt not found in {model_dir}")
        return False
    
    # Check size to ensure it's not just an LFS pointer
    size_mb = model_pt.stat().st_size / (1024**2)
    if size_mb < 1:
        print(f"   ❌ model.pt is only {size_mb:.3f} MB - appears to be an LFS pointer")
        print("   Please download the actual model weights first")
        return False
    
    print(f"   Model size: {size_mb:.1f} MB")
    
    # Check if ONNX files already exist
    onnx_files = {
        "model.onnx": output_dir / "model.onnx",
        "model_quant.onnx": output_dir / "model_quant.onnx",
        "model_eb.onnx": output_dir / "model_eb.onnx",
        "model_eb_quant.onnx": output_dir / "model_eb_quant.onnx"
    }
    
    existing = []
    for name, path in onnx_files.items():
        if path.exists():
            existing.append(name)
    
    if existing:
        print(f"   ⚠️  Found existing ONNX files: {', '.join(existing)}")
        response = input("   Overwrite existing files? (yes/no): ").lower().strip()
        if response not in ['yes', 'y']:
            print("   Skipping conversion")
            return True
    
    try:
        # Import FunASR export utilities
        from funasr import AutoModel
        
        print("   ⏳ Loading PyTorch model...")
        
        # Load the model
        model = AutoModel(model=str(model_dir))
        
        print("   ⏳ Converting to ONNX format...")
        
        # Export to ONNX - FunASR handles multiple ONNX variants
        # This will create model.onnx, model_quant.onnx, etc.
        model.export(
            type="onnx",
            quantize=True,  # Also create quantized version
            fallback_num=2,  # Number of fallback models
            output_dir=str(output_dir)
        )
        
        # Check which files were created
        created = []
        for name, path in onnx_files.items():
            if path.exists():
                size_mb = path.stat().st_size / (1024**2)
                created.append(f"{name} ({size_mb:.1f} MB)")
        
        if created:
            print(f"   ✅ Successfully created: {', '.join(created)}")
            return True
        else:
            print("   ❌ No ONNX files were created")
            return False
            
    except ImportError as e:
        print(f"   ❌ Missing required module: {e}")
        print("   Install with: pip install funasr[export]")
        return False
    except Exception as e:
        print(f"   ❌ Conversion failed: {e}")
        
        # Fallback: Try using onnx export directly
        print("\n   Trying alternative export method...")
        return convert_with_onnx_export(model_dir, output_dir, "paraformer")


def convert_punctuation_to_onnx(model_dir: Path, output_dir: Optional[Path] = None):
    """Convert Punctuation model to ONNX format.
    
    Args:
        model_dir: Directory containing the PyTorch model
        output_dir: Output directory for ONNX model (default: same as model_dir)
    """
    print(f"\n📦 Converting Punctuation model to ONNX")
    print(f"   Source: {model_dir}")
    
    if output_dir is None:
        output_dir = model_dir
    
    # Check if model.pt exists
    model_pt = model_dir / "model.pt"
    if not model_pt.exists():
        print(f"   ❌ model.pt not found in {model_dir}")
        return False
    
    # Check size
    size_mb = model_pt.stat().st_size / (1024**2)
    if size_mb < 1:
        print(f"   ❌ model.pt is only {size_mb:.3f} MB - appears to be an LFS pointer")
        print("   Please download the actual model weights first")
        return False
    
    print(f"   Model size: {size_mb:.1f} MB")
    
    # Check if ONNX files already exist
    onnx_files = {
        "model.onnx": output_dir / "model.onnx",
        "model_quant.onnx": output_dir / "model_quant.onnx"
    }
    
    existing = []
    for name, path in onnx_files.items():
        if path.exists():
            existing.append(name)
    
    if existing:
        print(f"   ⚠️  Found existing ONNX files: {', '.join(existing)}")
        response = input("   Overwrite existing files? (yes/no): ").lower().strip()
        if response not in ['yes', 'y']:
            print("   Skipping conversion")
            return True
    
    try:
        # Import FunASR export utilities
        from funasr import AutoModel
        
        print("   ⏳ Loading PyTorch model...")
        
        # Load the model
        model = AutoModel(model=str(model_dir))
        
        print("   ⏳ Converting to ONNX format...")
        
        # Export to ONNX
        model.export(
            type="onnx",
            quantize=True,  # Also create quantized version
            output_dir=str(output_dir)
        )
        
        # Check which files were created
        created = []
        for name, path in onnx_files.items():
            if path.exists():
                size_mb = path.stat().st_size / (1024**2)
                created.append(f"{name} ({size_mb:.1f} MB)")
        
        if created:
            print(f"   ✅ Successfully created: {', '.join(created)}")
            return True
        else:
            print("   ❌ No ONNX files were created")
            return False
            
    except ImportError as e:
        print(f"   ❌ Missing required module: {e}")
        print("   Install with: pip install funasr[export]")
        return False
    except Exception as e:
        print(f"   ❌ Conversion failed: {e}")
        
        # Fallback: Try using onnx export directly
        print("\n   Trying alternative export method...")
        return convert_with_onnx_export(model_dir, output_dir, "punctuation")


def convert_with_onnx_export(model_dir: Path, output_dir: Path, model_type: str):
    """Fallback method to convert models using torch.onnx.export.
    
    Args:
        model_dir: Directory containing the PyTorch model
        output_dir: Output directory for ONNX model
        model_type: Type of model ("paraformer" or "punctuation")
    """
    try:
        import torch
        import torch.onnx
        import numpy as np
        
        print("   Using torch.onnx.export method...")
        
        # Load the model
        model_pt = model_dir / "model.pt"
        checkpoint = torch.load(model_pt, map_location="cpu")
        
        # The structure depends on the model type
        if isinstance(checkpoint, dict):
            if "model" in checkpoint:
                model = checkpoint["model"]
            elif "state_dict" in checkpoint:
                # Need to reconstruct model architecture
                print("   ❌ Model architecture reconstruction needed - not implemented")
                print("   Please use FunASR's export method instead")
                return False
            else:
                print(f"   ❌ Unknown checkpoint structure: {checkpoint.keys()}")
                return False
        else:
            model = checkpoint
        
        # Set to eval mode
        if hasattr(model, 'eval'):
            model.eval()
        
        # Create dummy input based on model type
        if model_type == "paraformer":
            # Paraformer expects audio features
            batch_size = 1
            seq_len = 100
            feat_dim = 80  # Typical mel-spectrogram dimension
            dummy_input = torch.randn(batch_size, seq_len, feat_dim)
        else:  # punctuation
            # Punctuation model expects token IDs
            batch_size = 1
            seq_len = 50
            vocab_size = 5000  # Approximate
            dummy_input = torch.randint(0, vocab_size, (batch_size, seq_len))
        
        # Export to ONNX
        output_path = output_dir / "model.onnx"
        print(f"   Exporting to {output_path}...")
        
        torch.onnx.export(
            model,
            dummy_input,
            str(output_path),
            export_params=True,
            opset_version=11,
            do_constant_folding=True,
            input_names=['input'],
            output_names=['output'],
            dynamic_axes={
                'input': {0: 'batch_size', 1: 'sequence'},
                'output': {0: 'batch_size', 1: 'sequence'}
            }
        )
        
        if output_path.exists():
            size_mb = output_path.stat().st_size / (1024**2)
            print(f"   ✅ Created model.onnx ({size_mb:.1f} MB)")
            
            # Try to create quantized version
            try:
                from onnxruntime.quantization import quantize_dynamic, QuantType
                
                quant_path = output_dir / "model_quant.onnx"
                print("   Creating quantized version...")
                
                quantize_dynamic(
                    str(output_path),
                    str(quant_path),
                    weight_type=QuantType.QInt8
                )
                
                if quant_path.exists():
                    quant_size_mb = quant_path.stat().st_size / (1024**2)
                    print(f"   ✅ Created model_quant.onnx ({quant_size_mb:.1f} MB)")
                    
            except ImportError:
                print("   ⚠️  Skipping quantization (onnxruntime not installed)")
            except Exception as e:
                print(f"   ⚠️  Quantization failed: {e}")
            
            return True
        else:
            print("   ❌ Failed to create ONNX file")
            return False
            
    except ImportError as e:
        print(f"   ❌ Missing required module: {e}")
        print("   Install with: pip install torch onnx onnxruntime")
        return False
    except Exception as e:
        print(f"   ❌ Alternative export failed: {e}")
        return False


def convert_all_funasr_models(models_dir: Optional[Path] = None):
    """Convert all FunASR models to ONNX format.
    
    Args:
        models_dir: Base directory for models (default: ~/.dora/models/asr)
    """
    if models_dir is None:
        asr_models_dir = os.getenv("ASR_MODELS_DIR")
        if asr_models_dir:
            models_dir = Path(asr_models_dir)
        else:
            models_dir = Path.home() / ".dora" / "models" / "asr"
    
    funasr_dir = models_dir / "funasr"
    
    if not funasr_dir.exists():
        print(f"❌ FunASR models directory not found: {funasr_dir}")
        print("   Please download models first using:")
        print("   python download_models.py --download funasr")
        return False
    
    print("\n" + "=" * 60)
    print("FunASR Model Converter (PyTorch → ONNX)")
    print("=" * 60)
    print(f"Models directory: {funasr_dir}\n")
    
    # Check FunASR installation
    if not check_funasr_installation():
        return False
    
    # Models to convert
    models = [
        {
            "name": "Paraformer ASR",
            "dir_name": "speech_seaco_paraformer_large_asr_nat-zh-cn-16k-common-vocab8404-pytorch",
            "converter": convert_paraformer_to_onnx
        },
        {
            "name": "Punctuation",
            "dir_name": "punc_ct-transformer_cn-en-common-vocab471067-large",
            "converter": convert_punctuation_to_onnx
        }
    ]
    
    success_count = 0
    for model_info in models:
        model_dir = funasr_dir / model_info["dir_name"]
        
        if not model_dir.exists():
            print(f"\n⚠️  {model_info['name']} model not found at {model_dir}")
            continue
        
        success = model_info["converter"](model_dir)
        if success:
            success_count += 1
    
    print("\n" + "=" * 60)
    print(f"Conversion Summary: {success_count}/{len(models)} models converted")
    print("=" * 60)
    
    if success_count == len(models):
        print("\n✅ All models converted successfully!")
        print("\nThe ONNX models provide:")
        print("  • Faster inference speed")
        print("  • Better cross-platform compatibility")
        print("  • Reduced memory usage (with quantized versions)")
        print("\nFunASR will automatically use ONNX models when available.")
    elif success_count > 0:
        print(f"\n⚠️  Partially successful: {success_count} models converted")
    else:
        print("\n❌ No models were converted")
    
    return success_count > 0


def list_onnx_models(models_dir: Optional[Path] = None):
    """List all ONNX models in the FunASR directory.
    
    Args:
        models_dir: Base directory for models
    """
    if models_dir is None:
        asr_models_dir = os.getenv("ASR_MODELS_DIR")
        if asr_models_dir:
            models_dir = Path(asr_models_dir)
        else:
            models_dir = Path.home() / ".dora" / "models" / "asr"
    
    funasr_dir = models_dir / "funasr"
    
    if not funasr_dir.exists():
        print(f"❌ FunASR models directory not found: {funasr_dir}")
        return
    
    print("\n📋 ONNX Models in FunASR Directory")
    print("=" * 60)
    
    total_count = 0
    total_size = 0
    
    for model_dir in funasr_dir.iterdir():
        if not model_dir.is_dir():
            continue
        
        onnx_files = list(model_dir.glob("*.onnx"))
        if onnx_files:
            print(f"\n📦 {model_dir.name}")
            for onnx_file in onnx_files:
                size_mb = onnx_file.stat().st_size / (1024**2)
                print(f"   • {onnx_file.name:25} {size_mb:8.1f} MB")
                total_count += 1
                total_size += size_mb
    
    if total_count > 0:
        print("\n" + "-" * 60)
        print(f"Total: {total_count} ONNX files, {total_size:.1f} MB")
    else:
        print("\nNo ONNX models found.")
        print("Use --convert to convert PyTorch models to ONNX format.")


def main():
    parser = argparse.ArgumentParser(
        description="Convert FunASR PyTorch models to ONNX format"
    )
    
    parser.add_argument(
        "--convert",
        type=str,
        choices=["all", "paraformer", "punctuation"],
        help="Convert models to ONNX format"
    )
    
    parser.add_argument(
        "--model-dir",
        type=str,
        help="Specific model directory to convert"
    )
    
    parser.add_argument(
        "--output-dir",
        type=str,
        help="Output directory for ONNX models (default: same as model directory)"
    )
    
    parser.add_argument(
        "--models-dir",
        type=str,
        help="Base models directory (default: ~/.dora/models/asr)"
    )
    
    parser.add_argument(
        "--list",
        action="store_true",
        help="List all ONNX models"
    )
    
    args = parser.parse_args()
    
    # Handle --list
    if args.list:
        models_dir = Path(args.models_dir) if args.models_dir else None
        list_onnx_models(models_dir)
        return
    
    # Handle --convert
    if args.convert:
        if args.convert == "all":
            models_dir = Path(args.models_dir) if args.models_dir else None
            success = convert_all_funasr_models(models_dir)
            sys.exit(0 if success else 1)
            
        elif args.model_dir:
            model_dir = Path(args.model_dir)
            output_dir = Path(args.output_dir) if args.output_dir else None
            
            if not model_dir.exists():
                print(f"❌ Model directory not found: {model_dir}")
                sys.exit(1)
            
            if args.convert == "paraformer":
                success = convert_paraformer_to_onnx(model_dir, output_dir)
            else:  # punctuation
                success = convert_punctuation_to_onnx(model_dir, output_dir)
            
            sys.exit(0 if success else 1)
        else:
            # Convert specific model type from default location
            models_dir = Path(args.models_dir) if args.models_dir else None
            if models_dir is None:
                asr_models_dir = os.getenv("ASR_MODELS_DIR")
                if asr_models_dir:
                    models_dir = Path(asr_models_dir)
                else:
                    models_dir = Path.home() / ".dora" / "models" / "asr"
            
            funasr_dir = models_dir / "funasr"
            
            if args.convert == "paraformer":
                model_dir = funasr_dir / "speech_seaco_paraformer_large_asr_nat-zh-cn-16k-common-vocab8404-pytorch"
                success = convert_paraformer_to_onnx(model_dir)
            else:  # punctuation
                model_dir = funasr_dir / "punc_ct-transformer_cn-en-common-vocab471067-large"
                success = convert_punctuation_to_onnx(model_dir)
            
            sys.exit(0 if success else 1)
    
    # Default: show help
    print("\nUsage examples:")
    print("\n  # Convert all FunASR models to ONNX:")
    print("  python convert_to_onnx.py --convert all")
    print("")
    print("  # Convert specific model type:")
    print("  python convert_to_onnx.py --convert paraformer")
    print("  python convert_to_onnx.py --convert punctuation")
    print("")
    print("  # Convert a specific model directory:")
    print("  python convert_to_onnx.py --convert paraformer --model-dir /path/to/model")
    print("")
    print("  # Convert with custom output directory:")
    print("  python convert_to_onnx.py --convert all --output-dir /path/to/output")
    print("")
    print("  # List all ONNX models:")
    print("  python convert_to_onnx.py --list")
    print("")
    print("Note: ONNX models provide faster inference and better compatibility.")
    print("      FunASR will automatically use ONNX models when available.")


if __name__ == "__main__":
    main()
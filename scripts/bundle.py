import os
import shutil
import zipfile
import subprocess

# Configuration
VERSION = "0.4.0"
TARGETS = ["x86_64-pc-windows-msvc", "x86_64-unknown-linux-gnu"]
DIST_DIR = "dist"

# NOTE: Pour un binaire 100% autonome incluant PHP, 
# téléchargez les versions "Portable" de PHP et placez-les dans un dossier 'bin/'.
# Le Gateway cherchera automatiquement 'php' dans son dossier local.

def build_gateway(target):
    print(f"📦 Building Gateway for {target}...")
    try:
        subprocess.run(["cargo", "build", "--release", "--target", target], cwd="gateway", check=True)
        return True
    except Exception as e:
        print(f"❌ Error building for {target}: {e}")
        return False

def create_bundle(target):
    target_name = target.split("-")[2] # windows, linux, etc
    bundle_name = f"nhtml-v{VERSION}-{target_name}"
    bundle_path = os.path.join(DIST_DIR, bundle_name)
    
    if os.path.exists(bundle_path):
        shutil.rmtree(bundle_path)
    os.makedirs(bundle_path)

    # 1. Copy Binary
    ext = ".exe" if "windows" in target else ""
    bin_path = f"gateway/target/{target}/release/gateway{ext}"
    shutil.copy(bin_path, os.path.join(bundle_path, f"nhtml{ext}"))

    # 2. Copy SDK & Polyfill
    shutil.copytree("sdk/php/src", os.path.join(bundle_path, "sdk/php"))
    shutil.copytree("gateway/counter/polyfill", os.path.join(bundle_path, "polyfill"))
    
    # 3. Copy Docs
    shutil.copy("docs/GUIDE_DEMARRAGE.md", os.path.join(bundle_path, "README.md"))
    shutil.copy("docs/SPEC.md", os.path.join(bundle_path, "SPEC.md"))

    # 4. Create ZIP
    zip_file = f"{bundle_path}.zip"
    with zipfile.ZipFile(zip_file, 'w', zipfile.ZIP_DEFLATED) as zipf:
        for root, dirs, files in os.walk(bundle_path):
            for file in files:
                abs_path = os.path.join(root, file)
                rel_path = os.path.relpath(abs_path, DIST_DIR)
                zipf.write(abs_path, rel_path)
    
    print(f"✅ Bundle created: {zip_file}")

if __name__ == "__main__":
    if not os.path.exists(DIST_DIR):
        os.makedirs(DIST_DIR)
    
    # For this POC, we build for the current host only to avoid cross-compilation setup issues
    import platform
    host_target = "x86_64-pc-windows-msvc" if platform.system() == "Windows" else "x86_64-unknown-linux-gnu"
    
    if build_gateway(host_target):
        create_bundle(host_target)

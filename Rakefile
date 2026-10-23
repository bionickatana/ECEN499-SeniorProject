# frozen_string_literal: true

require "rake"
require "fileutils"
require "rbconfig"

# ------------------------------------------------------------
# Project configuration
# ------------------------------------------------------------

PROJECT_NAME = "INL_temperature_nodes"

ROOT_DIR = File.expand_path(__dir__)
DEPS_DIR = File.join(ROOT_DIR, "deps")
SDK_DIR  = File.join(DEPS_DIR, "pico-sdk")
BUILD_DIR = File.join(ROOT_DIR, "build")

# Change this to "pico2" for an RP2350 / Raspberry Pi Pico 2.
# Change this to "pico" for an RP2040 / Raspberry Pi Pico.
BOARD = ENV.fetch("PICO_BOARD", "pico2")

SDK_REPO = "https://github.com/raspberrypi/pico-sdk.git"

# ------------------------------------------------------------
# Helpers
# ------------------------------------------------------------

def run!(*command)
  puts "\n> #{command.join(' ')}"
  sh(*command)
end

def tool_available?(tool)
  executable = RbConfig::CONFIG["host_os"] =~ /mswin|mingw|cygwin/ \
    ? "#{tool}.exe" \
    : tool

  ENV.fetch("PATH", "").split(File::PATH_SEPARATOR).any? do |path|
    File.executable?(File.join(path, executable))
  end
end

def require_tools!(*tools)
  missing = tools.reject { |tool| tool_available?(tool) }

  return if missing.empty?

  abort <<~MESSAGE
    Missing required tools: #{missing.join(", ")}

    Install the missing tools and run `rake setup` again.
    See the dependency instructions below the Rakefile.
  MESSAGE
end

def sdk_ready?
  File.file?(File.join(SDK_DIR, "external", "pico_sdk_import.cmake")) &&
    File.directory?(File.join(SDK_DIR, "lib", "tinyusb"))
end

def require_sdk!
  unless sdk_ready?
    abort "Pico SDK is not ready. Run `rake setup` first."
  end
end

def configure!
  require_sdk!

  run!(
    "cmake",
    "-S", ROOT_DIR,
    "-B", BUILD_DIR,
    "-DPICO_BOARD=#{BOARD}",
    "-DCMAKE_BUILD_TYPE=Debug"
  )
end

# ------------------------------------------------------------
# Default task
# ------------------------------------------------------------

desc "Set up dependencies and build the blink project"
task default: [:setup, :build]

# ------------------------------------------------------------
# Dependency setup
# ------------------------------------------------------------

desc "Check build tools and download the Pico SDK"
task :setup do
  require_tools!("git", "cmake", "ninja", "arm-none-eabi-gcc")

  FileUtils.mkdir_p(DEPS_DIR)

  unless File.directory?(File.join(SDK_DIR, ".git"))
    if File.exist?(SDK_DIR)
      abort "#{SDK_DIR} exists but is not a Git repository."
    end

    run!("git", "clone", "--depth", "1", SDK_REPO, SDK_DIR)
  end

  # Initialize SDK dependencies, including TinyUSB.
  run!("git", "-C", SDK_DIR, "submodule", "update",
       "--init", "--recursive")

  unless sdk_ready?
    abort "Pico SDK setup is incomplete. Check the SDK checkout."
  end

  puts "\nPico SDK is ready."
  puts "Target board: #{BOARD}"
end

# ------------------------------------------------------------
# Build tasks
# ------------------------------------------------------------

desc "Configure CMake and build the firmware"
task build: :setup do
  configure!

  run!("cmake", "--build", BUILD_DIR, "--parallel")

  puts "\nBuild complete!"
  puts "UF2: #{File.join(BUILD_DIR, "#{PROJECT_NAME}.uf2")}"
end

desc "Remove generated build files"
task :clean do
  FileUtils.rm_rf(BUILD_DIR)
  puts "Build directory removed."
end

desc "Clean and rebuild the firmware"
task rebuild: [:clean, :build]

# ------------------------------------------------------------
# Deployment
# ------------------------------------------------------------

desc "Flash the firmware using picotool"
task flash: :build do
  require_tools!("picotool")

  uf2 = File.join(BUILD_DIR, "#{PROJECT_NAME}.uf2")

  run!("picotool", "load", uf2, "-f")
  run!("picotool", "reboot")
end

desc "Print project configuration"
task :info do
  puts "Project: #{PROJECT_NAME}"
  puts "Board:   #{BOARD}"
  puts "Root:    #{ROOT_DIR}"
  puts "SDK:     #{SDK_DIR}"
  puts "Build:   #{BUILD_DIR}"
end

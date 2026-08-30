#!/usr/bin/env ruby
# frozen_string_literal: true

tag, sha256, output = ARGV

abort "usage: #{$PROGRAM_NAME} TAG SHA256 OUTPUT" unless output
abort "tag must be a stable semantic version such as v1.2.3" unless tag.match?(/\Av(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\.(?:0|[1-9]\d*)\z/)
abort "SHA-256 must contain 64 lowercase hexadecimal characters" unless sha256.match?(/\A[0-9a-f]{64}\z/)

formula = <<~RUBY
  class CodeACv < Formula
    desc "Build CVs from Markdown or structured data"
    homepage "https://github.com/voxvanhieu/code-a-cv"
    url "https://github.com/voxvanhieu/code-a-cv/releases/download/#{tag}/source.tar.gz"
    sha256 "#{sha256}"
    license "MIT"
    head "https://github.com/voxvanhieu/code-a-cv.git", branch: "main"

    depends_on "rust" => :build

    def install
      system "cargo", "install", "--profile=dist", *std_cargo_args(path: "crates/cac")
    end

    test do
      system bin/"cac", "init"
      assert_path_exists testpath/"cv.md"

      system bin/"cac", "build", "cv.md", "--format", "html", "--output", "dist"
      assert_path_exists testpath/"dist/cv.html"
    end
  end
RUBY

File.write(output, formula)

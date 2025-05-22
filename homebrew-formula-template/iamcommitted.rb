class Iamcommitted < Formula
  desc "AI micro bot for generating Git commit messages"
  homepage "https://github.com/darkin100/iamcommitted"
  url "https://github.com/darkin100/iamcommitted/releases/download/v0.1.0/iamcommitted-v0.1.0-macos.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_FIRST_RELEASE"
  version "0.1.0"
  license "MIT" # Update with your actual license

  def install
    bin.install "iamcommitted"
  end

  test do
    assert_match "iamcommitted #{version}", shell_output("#{bin}/iamcommitted --version")
  end

  def caveats
    <<~EOS
      This application requires an OpenAI API key to function.
      Please set the OPENAI_API_KEY environment variable:
        export OPENAI_API_KEY="your_api_key_here"
    EOS
  end
end

file_name="$1"
upload_prefix="$2"

git_commit_shash=$(git rev-parse --short "$GITHUB_SHA")
git_branch=$(echo $GITHUB_REF | cut -d'/' -f 3)
curl -H 'Content-Type: multipart/form-data' \
  -X POST \
  -F "payload_json={\"content\": \"Automated release build for $git_commit_shash on $git_branch\"}" \
  -F "file=@$file_name; filename=${upload_prefix}_${git_commit_shash}.exe" $DISCORD_RELEASE_WEBHOOK
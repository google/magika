# How to Submit This Pull Request

## ⚠️ Important Steps Before Submitting

### 1. Fork the Repository
If you haven't already:
```bash
# Go to https://github.com/google/magika
# Click "Fork" button in top-right
```

### 2. Add Your Fork as Remote
```bash
cd /tmp/magika

# Add your fork (replace YOUR_USERNAME)
git remote add fork https://github.com/YOUR_USERNAME/magika.git

# Verify remotes
git remote -v
```

### 3. Push the Branch
```bash
# Push to your fork
git push fork windows-gui
```

### 4. Create Pull Request

1. Go to your fork: `https://github.com/YOUR_USERNAME/magika`
2. You'll see a banner: "Compare & pull request" - Click it
3. Or go to: `https://github.com/google/magika/compare`
4. Select: `base: main` ← `compare: YOUR_USERNAME:windows-gui`

### 5. Fill in PR Details

Use the content from `PR_DESCRIPTION.md` or `PULL_REQUEST_TEMPLATE.md`

**Title:**
```
Add Windows GUI application for Magika
```

**Description:**
Copy content from `PR_DESCRIPTION.md` or customize as needed.

### 6. Sign the CLA

When you submit the PR, Google's CLA bot will comment asking you to sign the Contributor License Agreement (CLA).

1. Follow the link in the bot's comment
2. Sign the CLA (it's quick)
3. Comment on the PR: `@googlebot I signed it!`

### 7. Wait for Review

The Magika maintainers will review your PR. Be prepared to:
- Answer questions
- Make requested changes
- Add screenshots (if requested)
- Run additional tests

## 📋 Pre-Submission Checklist

Before pushing, verify:

- [ ] All commits have descriptive messages
- [ ] Code follows project style (Google Python style)
- [ ] Documentation is complete
- [ ] No sensitive information in commits
- [ ] Branch is up-to-date with main
- [ ] All files have proper copyright headers
- [ ] No unnecessary files included

## 🔍 Review Your Changes

```bash
# See all changes
git diff main

# See file statistics
git diff main --stat

# See commit history
git log main..windows-gui --oneline

# Review each commit
git log -p
```

## 🐛 If Something Goes Wrong

### Forgot to sign commits?
```bash
# Sign the last commit
git commit --amend -S

# Force push (careful!)
git push fork windows-gui --force
```

### Need to update from upstream?
```bash
# Add upstream remote
git remote add upstream https://github.com/google/magika.git

# Fetch latest
git fetch upstream

# Rebase your branch
git rebase upstream/main

# Force push to your fork
git push fork windows-gui --force
```

### Made a mistake in commit?
```bash
# Edit last commit
git commit --amend

# Interactive rebase to edit older commits
git rebase -i main
```

## 📧 Communication Tips

When maintainers comment:

- ✅ Respond promptly and politely
- ✅ Ask for clarification if needed
- ✅ Make requested changes quickly
- ✅ Test changes before pushing
- ✅ Mark conversations as "resolved" when done
- ❌ Don't take feedback personally
- ❌ Don't argue without technical justification
- ❌ Don't force-push without warning reviewers

## 🎯 What to Expect

### Timeline
- **Initial Response**: 1-7 days
- **Review Rounds**: 1-3 iterations typically
- **Merge Time**: 1-4 weeks total (varies)

### Possible Outcomes
1. **Approved & Merged** ✅
   - Your code becomes part of Magika!
   
2. **Changes Requested** 🔄
   - Make changes and push again
   - Reviewers will re-review
   
3. **Needs Discussion** 💬
   - Design decisions may need debate
   - Be open to alternative approaches
   
4. **Closed (Not Right Fit)** ❌
   - Don't be discouraged!
   - Ask for feedback on why
   - Consider maintaining a fork

## 📸 Screenshots

If requested, add screenshots:

1. Take screenshots of the GUI
2. Upload to an image hosting service (or GitHub)
3. Add to PR description:
   ```markdown
   ## Screenshots
   
   ![Magika GUI](https://your-image-url.png)
   ```

## 🎓 Additional Resources

- [Google's CLA](https://cla.developers.google.com/)
- [How to Write a Git Commit Message](https://chris.beams.io/posts/git-commit/)
- [Google Open Source Guidelines](https://opensource.google/conduct/)
- [GitHub PR Best Practices](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests)

## 🚀 Ready to Submit?

1. Fork the repo
2. Push your branch: `git push fork windows-gui`
3. Create PR on GitHub
4. Sign the CLA
5. Wait for review
6. 🎉 Celebrate your contribution!

---

Good luck! Your contribution will help make Magika more accessible to Windows users! 🙌

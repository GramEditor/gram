---
name: 'Issue'
about: 'Instructions for how to file an issue'
title: 'Summary of the issue'
ref: 'main'
labels:
  - bug
---

Wiki: https://codeberg.org/GramEditor/gram/wiki
Docs: https://gram-editor.com/docs/
Chat: https://slidge.im/gram/#/guest?join=gram@rooms.slidge.im

This issue tracker is mainly for tracking and dealing with bugs. There is no big
team working on Gram so don't be surprised if issues and feature requests end up
sitting around without much movement.

- If you want to make a feature request or have an idea for improvements, feel
  free to do so. We tried to steer people to the Wiki for such things but
  creating an issue is probably an easier way to track them long term. Be
  prepared to wait unless you are interested in working on the feature yourself :)

- If you have a question about how the software works or how to configure it,
  check the wiki guides or the documentation, or join the chatroom and ask there
  first. If the question turns out to be a missing feature, create a feature
  request. If it's a bug, come back and create an issue.


If your issue really is a bug that you've found:

- Is it caused by hardware, for example missing support for your particular
  graphics device? If so, you are probably the person best suited to solve the
  problem since you are the one able to reproduce it. Start by trying to work out
  as much about the cause of the issue yourself, and include as much detail as you
  can in the issue. See the guide to debugging crashes in the documentation for
  more information on retrieving logs or using a debugger with Gram.

- Is it a packaging problem, like missing builds for Windows, missing packages
  for your architecture or support needed for a particular Linux distribution?
  Just like in the previous case, you are probably the one most able to fix the
  problem yourself. There is a mirror of the repository on Github only so that we
  can run Github Actions on platforms that we don't have runners for on Codeberg.
  Contributions to improve the workflows for both Forgejo and Github are welcome.


For any other issues, try to investigate the problem before creating an issue
and try to provide relevant information (see the documentation for details on
how to get logs from the editor, but also tell us which OS, version and
distribution you are on).

Thank you <3

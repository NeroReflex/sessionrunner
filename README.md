# SessionRunner

This software implements service management and session swiching within a logged-in user.

It can act as both:
  - implement game mode/desktop session switching as handhelds distro does
  - replacement of systemd user slice for very simple tasks in embedded devices

## Why?

Because currently the autostart of GUI sessions is a mess:
  - start weston from systemd directly: the user won't be logged in or you will need to use root user
  - session switching performs a logout and a new login: ugly, clogger logs, easy to break
  - session switching must depend on how you logged in: the whole distro must be specialized

## How To

Using SessionRunner is easy enough: configure you login software (can be gdm, sddm or any other)
to run SessionRunner when the user has performed the login.

Exact configurations steps depends on the method used to login.

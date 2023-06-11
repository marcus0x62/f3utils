# Setup

 In order to use this, you'll need a web server that can host the
[report page](html/qsignup_report.html).  You'll also need to determine the team id Qsignups uses for
your region by querying the Paxminer database.  There isn't a direct region-to-team-id mapping table in
the Paxminer database, but you can deduce it from the AO table.  Query against it for one of your AOs
and note the team_id column.  In the example below, I'm using our AO 'The Battleground'.

```
% mysql -u paxminer -p -h f3stlouis.cac36jsyb5ss.us-east-2.rds.amazonaws.com -D f3stcharles
Enter password:
Reading table information for completion of table and column names
You can turn off this feature to get a quicker startup with -A

Welcome to the MySQL monitor.  Commands end with ; or \g.
Your MySQL connection id is 1522500
Server version: 8.0.28 Source distribution

Copyright (c) 2000, 2023, Oracle and/or its affiliates.

Oracle is a registered trademark of Oracle Corporation and/or its
affiliates. Other names may be trademarks of their respective
owners.

Type 'help;' or '\h' for help. Type '\c' to clear the current input statement.

mysql> SELECT ao_display_name, ao_location_subtitle, team_id FROM qsignups_aos WHERE ao_display_name LIKE 'The Battleground';
+------------------+--------------------------------------------------------+-------------+
| ao_display_name  | ao_location_subtitle                                   | team_id     |
+------------------+--------------------------------------------------------+-------------+
| The Battleground | One Franklin Park
6100 Tower Circle
Franklin, TN 37067 | T3SACDW4S   |
| The Battleground | Oakville Middle School                                 | T0114FM4E9W |
+------------------+--------------------------------------------------------+-------------+
2 rows in set (0.06 sec)

mysql>

```

 *note that AO names can be duplicated -- be sure the description in the ao_location_subtitle column
 makes sense.*

Take the value from the team_id column and update the team_id variable in
[report](html/qsignup_report.html).  It is located near the top of the file:

```
<html>
  <head>
    <title>QSignups Report</title>
    <script language="JavaScript">
      var create_ao_list = true;
      *var team_id = 'T3SACDW4S';* // You need to query this from the Paxminer database.
```

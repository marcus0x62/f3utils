# Setup - Web Page

 In order to use this, you'll need a web server that can host the
[report page](html/qsignup_report.html).  You'll also need to determine the team id Qsignups uses for
your region by querying the Paxminer database.  There isn't a direct region-to-team-id mapping table in
the Paxminer database, but you can deduce it from the AO table.  Query against it for one of your AOs
and note the team_id column.  In the example below, I'm using our AO 'The Battleground'.

```
% mysql -u paxminer -p -h f3stlouis.cac36jsyb5ss.us-east-2.rds.amazonaws.com -D f3stcharles
Enter password:

Output elided...

mysql> SELECT ao_display_name, ao_location_subtitle, team_id FROM qsignups_aos WHERE ao_display_name LIKE 'The Battleground';
+------------------+--------------------------------------------------------+-------------+
| ao_display_name  | ao_location_subtitle                                   | team_id     |
+------------------+--------------------------------------------------------+-------------+
| The Battleground | One Franklin Park 6100 Tower Circle Franklin, TN 37067 | T3SACDW4S   |
| The Battleground | Oakville Middle School                                 | T0114FM4E9W |
+------------------+--------------------------------------------------------+-------------+
2 rows in set (0.06 sec)

mysql>
```

**note that another region might also use your AO name -- be sure the description in the
ao_location_subtitle column makes sense.**

Take the value from the team_id column and update the team_id variable in
[report](html/qsignup_report.html).  It is located near the top of the file:

```
<html>
  <head>
    <title>QSignups Report</title>
    <script language="JavaScript">
      var create_ao_list = true;
      var team_id = 'T3SACDW4S'; // You need to query this from the Paxminer database.
```

After that, all you should have to do is copy the file to your web server and let your PAX know the URL.

# Setup - Lambda Function

If you want to customize the Lambda function, retrieve different data from the database, or just want to
host your own copy for whatever reason, you'll need to run the [deployment script](deploy.sh).  You
should also inspect the [Cloudformation Template](template.yml) -- if you are hosting your own
paxminer/qsignups database, the hostname, username, and database name will need to be updated to your
values.  If you're using the hosted versions by Beaker and Moneyball, the values are already set
correctly.

```% ./deploy.sh

Successfully packaged artifacts and wrote output template to file .out.yml.
Execute the following command to deploy the packaged template
aws cloudformation deploy --template-file /Users/marcusb/src/f3utils/qsignup_report/.out.yml --stack-name <YOUR STACK NAME>

Waiting for changeset to be created..

No changes to deploy. Stack nodejs-apig is up to date
*** Created/updated Lambda function nodejs-apig-function-XXXXXXXXXX, which can be reached at https://example-xyz.execute-api.us-east-2.amazonaws.com/api/
```

The lambda function retrieves the database password from the Systems Manager Parameter Store service.
Before invoking the function, add the password (if you don't have the password for the read only
paxminer account, contact Beaker on F3 Nation Slack.)

``` % aws ssm put-parameter --type SecureString --name /qsignup_report/dbPassword --value <password>
{
    "Version": 1,
    "Tier": "Standard"
}
(END)
%```

# acrlogin
Allows you to connect to an acr registry using a service principal and secret stored in Azure Key Vault.
Before running the tool make sure you are logged in to Azure CLI so that the tool has access to the Azure Key Vault.

The motivation for the tool comes from Microsoft internal workflows where all containers build in official pipeline are 
pushed to a secure tenant which are not accessible by developers. Each developer is expected to setup a service principal,
grant access to the ACR for the service principal and then use it. This process is very hard to scale.

With this tool you have to setup the permission once and everything will work seamlessly without much maintenance.

## Setup
1. Register a domain with certificate authority of your choice.
2. Create an app in the tenant(eg: f4740579-2207-4615-a076-746e501c7314) and set it up to use subject name validation. Setting up subject name validation typically requires adding the following section to the manifest: <code>
    "trustedCertificateSubjects": [
        {
            "authorityId": "00000000-0000-0000-0000-000000000001",
            "subjectName": "test.azure-test.net"
        }
    ]
</code> Assuming your app id is: <code>e7439801-24f9-4583-a3f3-f2e85ab5e8ff</code>
3. Setup a keyvault(eg: AcrLoginKeyVault) with a certificate (eg: AppCert) that has the certificate above as the subject name and is created by the right certificate authority.
4. Assign reader access to relevant people and groups to the above certs. Its easy to setup a team security group and then give the get permissions to the entire group.
5. Grant the app reader permissions on the ACR.
6. Now everyone who has access to the cert can login to the acr using the following command: <code>acrlogin.exe --vault-name AcrLoginKeyVault --certificate-name AppCert --tenant-id f4740579-2207-4615-a076-746e501c7314 --client-id e7439801-24f9-4583-a3f3-f2e85ab5e8ff -u --acr-name #acr-name# </code>

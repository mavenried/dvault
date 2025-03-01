
<h1 align="center">DVault</h1>
<p align="center">Create and manage \"vaults\"</p>

## What is DVault?
DVault is an encryped folder manager written in rust.
It was built as an intoduction to aes encryption.

## Usage
### setup
```sh
dvault setup <path_to_dvault_home>
```

This command needs to be run to set where the encrypted folders are located.  

`recommended: ~/DVault`
>Without any arguments, this command will return the home path instead.

### new

```sh
dvaut new <name>
```
Used to create an encrypted folder (vault).  

>You will be asked to set a password for the vault.

### lock & unlock
```sh 
dvault lock <name>
dvault unlock <name>
```
These commands lock and unlock vaults respectively. Locked Vaults are encrypted and stored in the `.dvault` directory. 
>Locking will delete the directory in the home path.

### list
```sh
dvault list
```
This command will list the names and the lock status of all existing vaults.

## What is the point of this?
This is simply a project coded up on a weekend. All because of a whim to start writing a diary.

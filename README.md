# What are Melda Tools?

Melda Tools are command line utilities to work with [libmelda](https://github.com/slashdotted/libmelda/).

# What is Melda?

Melda is a Delta-State JSON CRDT. CRDTs, which stand for Conflict-free Replicated Data Types, are data structures which can be replicated (copied) across multiple computers in a network. Each replica can be individually and concurrently updated without the need for central coordination or synchronization. Updates made on each replica can be merged at any time.

There exist different types of CRDTs: operation-based CRDTs (which generate and exchange update operations between replicas), state-based CRDTS (which exchange and merge the full state of each replica) and delta-state CRDT, such as Melda, (which exchange only the differences between versions, or states, of the data type).

Melda natively supports the JSON data format and provides a way to synchronize changes made to arbitrary JSON documents.

# How do I use Melda Tools?

You need to clone this repository and then compile the CLI tool executable:
```
git clone https://github.com/slashdotted/libmelda-tools
cd libmelda-tools
cargo build
```
To understand how to use Melda using the CLI tools, we consider the following situation, where a shared JSON document used by a fictitious activity planning software (i.e. a todo management software) is concurrently updated by multiple parties. The provided JSON is generated by the application (by serializing its data model). We assume that user **Alice** creates the first version of the shared JSON document, which will be named **v1_alice.json**. This first version contains the following data:

```json
{
	"software" : "MeldaDo",
	"version" : "1.0.0",
	"items♭" : []
}

```
The **root** object contains three fields: a **software** field which defines the name of the application, a **version** field, which sets the version of the software, and an **items♭** field, which maps to an array of JSON objects (one for each todo). Since this is the first version, the array of items is empty. The **♭** suffix is used to ask Melda to *flatten* the contents of the array, by extracting the contained JSON objects in order to keep track of their changes individually.

To better understand the purpose of the *flattening* procedure, consider how Melda processes the following two JSON files. The first one, named **v2_alice_noflat.json** contains:

```json
{
	"software" : "MeldaDo",
	"version" : "1.0.0",
	"items" : [
	   {"_id" : "alice_todo_01", "title" : "Buy milk", "description" : "Go to the grocery store"}
	]
}

```
In this case, Melda will keep the root object as is, and the changes made to the items array by one user will not merge with changes made by other users. So, for example, if two users add an element to the array on their replica and later merge those replicas, only one of the elements will be visible. On the contrary, consider now another version of the document, named **v2_alice.json**, which contains:

```json
{
	"software" : "MeldaDo",
	"version" : "1.0.0",
	"items♭" : [
	   {"_id" : "alice_todo_01", "title" : "Buy milk", "description" : "Go to the grocery store"}
	]
}

```
In this case the object within the **items♭** array will be extracted and tracked individually. In particular, two JSON objects results from the above document:
```json
{
	"_id" : "√",
	"software" : "MeldaDo",
	"version" : "1.0.0",
	"items♭" : [
	  "alice_todo_01"
	]
}

```
And the todo item itself:
```json
{
	"_id" : "alice_todo_01",
	"title" : "Buy milk",
	"description" : "Go to the grocery store"
}

```
Please notice that each object has its own unique identifier stored in the **_id** field. If an identifier is not provided by the client application, Melda will auto-generate one. The root object is always identified by **√** (this identifier cannot be changed by the client application). Since each object of the **items♭** array is tracked individually, if an user adds an element to the array and later merges his/her replica with another user all changes will be preserved.

If the collection of items becomes too large we can ask Melda to only store difference arrays between the newest revision of the document and the previous one. For that we simply need to prefix the key of the **items** field with the Δ character (greek capital letter delta). Version **delta_alice.json** might therefore become:
```json
{
	"software" : "MeldaDo",
	"version" : "1.0.0",
	"Δitems♭" : [
	   {"_id" : "alice_todo_01", "title" : "Buy milk", "description" : "Go to the grocery store"}
	]
}

```
To keep things simple, in the following we will not use difference arrays. Let's go back to our example situation...
Up until this point we only considered some JSON data, but we have yet to see how we can interact with Melda in order to update the data structure.

## Adapters

Melda implements a modular design where the logic of the CRDT is separated from the data storage. Storing the data (in our case, delta states) is achieved by means of **Adapters**. Melda already provides different types of adapters, supporting in-memory storage (**MemoryAdapter**), filesystem storage (**FilesystemAdapter**) and Solid Pods (**SolidAdapter**). Furthermore, it is possible to use a meta-adapter to compress data using the Flate2 algorithm (**Flate2Adapter**): such an adapter can be composed with other adapters.

With Melda Tools we can choose an an adapter that will store data on the filesystem (in the **todolist** directory) by specifying a path like **file://$(pwd)/todolist** (where *$(pwd)* returns the absolute path of the current directory). If we want to used compression we would add the **Flate2Adapter** we would use **file+flate://$(pwd)/todolist**.


## Creating the CRDT

In order to create the first state or update the state of the CRDT we use the **update** command. Suppose that the first version of the document is stored in file **v1.json** and contains:
```json
{ "software" : "MeldaDo", "version" : "1.0.0", "items♭" : []}
```
Alice can create / update the CRDT inside the **todolist** directory with:
```bash
./target/debug/libmelda-tools update -a "Alice" -d "First commit" -j v1.json -t file://$(pwd)/todolist
```
Please note that we assume that **Melda Tools** have been compiled in debug mode and that the executable is **./target/debug/libmelda-tools**.

Updates made to the CRDT are committed to disk. We can pass an optional **author** (*-a* option) and **description** (*-d* option) with additional information that will be stored along with the updates.

The result of the **update** is either an error message or the identifier of the committed block.

Upon success, on disk (in the **todolist** directory) the following content should have been created:
```
todolist/
├── 49
│   └── 49ccea4d5797250208edf9bc5d0b89edf23c30a61f5cb3fafb87069f07276a62.delta
└── b4
    └── b4e50e445542c4737f4cfd7a9193ffd3be3794049d361d114a44f36434257cb3.pack
```

The **.delta** file is called **delta block**, and contains the versioning information of each object in the CRDT, wherease the **.pack** file is the **data pack** which stores the actual JSON content of each object. Each commit produces a new delta block (with a different name, which corresponds to the hash digest of its content) and possibly a data pack (if new JSON values are produced). The directory structure of the **todolist** directory organizes files into sub-directories according to their prefix. 

Alice can perform another update using (again) the **update** method. First, the contents of a new version are stored in **v2.json**:
```json
{ "software" : "MeldaDo", "version" : "1.0.0", "items♭" : [
       {"_id" : "alice_todo_01", "title" : "Buy milk", "description" : "Go to the grocery store"}
    ]
    }
```
Then the CRDT is updated and changes are committed:
```bash
./target/debug/libmelda-tools update -a "Alice" -d "Add buy milk" -j v2.json -t file://$(pwd)/todolist
```

The changes will reflect on disk (with new packs and blocks created in the corresponding directories):
```
todolist/
├── 2b
│   └── 2b0a463fcba92d5cf7dae531a5c40b67aaa0f45ab351c15613534fb5bba28564.pack
├── 49
│   └── 49ccea4d5797250208edf9bc5d0b89edf23c30a61f5cb3fafb87069f07276a62.delta
├── b4
│   └── b4e50e445542c4737f4cfd7a9193ffd3be3794049d361d114a44f36434257cb3.pack
└── b6
    └── b6297035f06f13186160577099759dea843addcd1fbd05d24da87d9ac071da3b.delta
```
## Reading the data

At any time it is possible to read the state of the CRDT back into a JSON document using the **read** command:
```bash
./target/debug/libmelda-tools read -s file://$(pwd)/todolist
```

This will print the following on the terminal:
```json
{"_id":"√","items♭":[{"_id":"alice_todo_01","description":"Go to the grocery store","title":"Buy milk"}],"software":"MeldaDo","version":"1.0.0"}
```

Each object managed by Melda will contain the **_id** field with the corresponding unique identifier.

## Sharing data

We now suppose that Alice shares the current state of the  **todolist** directory with Bob (she can simply zip the contents and send the compressed file by e-mail to Bob). We assume that Bob saves the contents in the **todolist_bob** directory. Bob can perform some updates (which we assume are stored in **v3_bob.json**):
```json
{ "software" : "MeldaDo", "version" : "1.0.0", "items♭" : [
       {"_id" : "alice_todo_01", "title" : "Buy milk", "description" : "Go to the grocery store"},
       {"_id" : "bob_todo_01", "title" : "Pay bills", "description" : "Withdraw 500 to pay bill"},
       {"_id" : "bob_todo_02", "title" : "Call mom", "description" : "Call mom to schedule dinner"}
    ]
    }
```

Bob updates his own replica with:
```bash
./target/debug/libmelda-tools update -a "Bob" -d "Add some todos" -j v3_bob.json -t file://$(pwd)/todolist_bob
```

As you might notice, two new items have been added by Bob. In the meantime, Alice continues to work on her replica, by removing one item (**alice_todo_01**) and adding a new item (**alice_todo_02**). The file used by Alice is called **v3_alice.json** and contains the following:
```json
{ "software" : "MeldaDo", "version" : "1.0.0", "items♭" : [
        {"_id" : "alice_todo_02", "title" : "Take picture of our dog", "description" : "It must be a nice one"}
     ]
     }
```

To update her own copy, Alice uses the following command line:
```bash
./target/debug/libmelda-tools update -a "Alice" -d "Some more stuff to do" -j v3_alice.json -t file://$(pwd)/todolist
```

Finally, Bob shares his own copy with Alice: now Alice simply needs to merge the content of the directory (as received from Bob) with the local directory (using something like **cp -r todolist_bob/* todolist/**). Alternatively, suppose that the data modified by Bob is in the **todolist_bob** directory on Alice's computer. To merge changes back into the **todolist** directory, Alice can use the **meld** method:
```bash
./target/debug/libmelda-tools meld -t file://$(pwd)/todolist -s file://$(pwd)/todolist_bob
```
Alice can then read the new state of the CRDT with:
```bash
./target/debug/libmelda-tools read -s file://$(pwd)/todolist
```

The result, printed on the terminal should look like:
```json
{"_id":"√","items♭":[{"_id":"bob_todo_01","description":"Withdraw 500 to pay bill","title":"Pay bills"},{"_id":"bob_todo_02","description":"Call mom to schedule dinner","title":"Call mom"},{"_id":"alice_todo_02","description":"It must be a nice one","title":"Take picture of our dog"}],"software":"MeldaDo","version":"1.0.0"}
```

As you can see, there is only one todo from Alice, as well as the two todos added by Bob.

Both Alice and Bob can see the history of changes made to their replica using the **log** command:
```bash
./target/debug/libmelda-tools log -s file://$(pwd)/todolist
```
For Alice the result will look like:
```
(A) Block: d0d23eeaf013b216a32386e708fb37489743cb2c9c8153082fc8e944a91eedf6
		Information: {"author":"Bob","description":"Add some todos"}
		Packs: ["515ebf5ebd96fe8210945856d09b53fa673434291a598c893db76bed117b243e"]
		Parents: ["460b4dd46257efbb018201d9c1ada3e165174241b8ef9a30f8f0f0b77a551283"]
(A) Block: ec11159e3497a89d1f0cb23db2600239535c70cc35a4f4b5a96e1d561d2bead3
		Information: {"author":"Alice","description":"Some more stuff to do"}
		Packs: ["967e769c2b65c0a30a9aeed1350ed78c46e98073c61a23421e8a7c4b721e61d0"]
		Parents: ["460b4dd46257efbb018201d9c1ada3e165174241b8ef9a30f8f0f0b77a551283"]
(-) Block: 460b4dd46257efbb018201d9c1ada3e165174241b8ef9a30f8f0f0b77a551283
		Information: {"author":"Alice","description":"Add buy milk"}
		Packs: ["2b0a463fcba92d5cf7dae531a5c40b67aaa0f45ab351c15613534fb5bba28564"]
		Parents: ["49ccea4d5797250208edf9bc5d0b89edf23c30a61f5cb3fafb87069f07276a62"]
(O) Block: 49ccea4d5797250208edf9bc5d0b89edf23c30a61f5cb3fafb87069f07276a62
		Information: {"author":"Alice","description":"First commit"}
		Packs: ["b4e50e445542c4737f4cfd7a9193ffd3be3794049d361d114a44f36434257cb3"]
```
The list of delta blocks contains **origin** blocks (**(O)**) and **anchor** blocks (**(A)**). Origin blocks are the first one that have been created: in our scenario there is only one origin, since the CRDT was created on one replica only (by Alice). There are however two **anchor** blocks, namely *d0d23eeaf013b216a32386e708fb37489743cb2c9c8153082fc8e944a91eedf6* (created by Bob) and *ec11159e3497a89d1f0cb23db2600239535c70cc35a4f4b5a96e1d561d2bead3* (created by Alice). This is due to the merge/meld operation that was performed by Alice. Multiple anchors will be referenced by the next commit.

Concurrent modifications made by Alice and Bob also resulted in a conflict. By default this is automatically hidden, since Melda can cope with this situation without problems. We can nonetheless show the conflicting information using the **conflicts** command:
```bash
./target/debug/libmelda-tools conflicts -s file://$(pwd)/todolist
```
This will show that the root document (√) has a conflict, and the conflicting versions will be shown (the one with 🏆 is the version currently chosen by Melda as the *winner*, conflicts are shown with 🗲):
```
√:
	🏆 3-8f147f811da66dccc212b3147a185c7c68d365e02ae84614e6533b7857d4744a_6258b20: {"items♭":["alice_todo_01","bob_todo_01","bob_todo_02","alice_todo_02"],"software":"MeldaDo","version":"1.0.0"}
	🗲 3-5bf6651423be2df90bf3a7250a5b8d7e457da397ab7a31bd24f96c099c183711_6258b20: {"items♭":["alice_todo_02","alice_todo_01","bob_todo_01","bob_todo_02"],"software":"MeldaDo","version":"1.0.0"}

```
Further updates will always consider the *winner*. We can however resolve the conflict (and make it disappear from the conflict view) using the **resolve** command:
```bash
./target/debug/libmelda-tools resolve -t file://$(pwd)/todolist
```
This command by default resolves all conflicts in all objects using the current *winner*. Different strategies can be chosen, in order to promote a different *winner*. The **conflicts** command will confirm that there are no conflicts.

# Publications

## 2022
Amos Brocco "Melda: A General Purpose Delta State JSON CRDT". 9th Workshop on Principles and Practice of Consistency for Distributed Data (PaPoC'22). April 2022. (Accepted)

## 2021
Amos Brocco "Delta-State JSON CRDT: Putting Collaboration on Solid Ground". (Brief announcement). 23rd International Symposium on Stabilization, Safety, and Security of Distributed Systems (SSS 2021). November 2021. 

# License
(c)2021-2022 Amos Brocco,
GPL v3 (for now... but I will evaluate a change of license - to something like BSD3/MIT/... in the near future)

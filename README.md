# libmelda-tools
Tools for [libmelda](https://github.com/slashdotted/libmelda/) 

# Examples

Create the first version of a JSON document (named **v1.json**):
```json
{
	"data" : [1,2,3,4,5,6],
	"version" : "1.0"
}
```
Using the **update** function, initialize the CRDT:
```bash
./target/debug/libmelda-tools update -a "Amos" -d "First commit" -j v1.json -t file://$(pwd)/docdoc
```
Now create the second version of the JSON document (named **v2.json**):
```json
{
	"data" : [1,2,3,4,5],
	"value" : "Some string",
	"version" : "2.0"
}
```
Update again the CRDT:
```bash
./target/debug/libmelda-tools update -a "Amos" -d "Second commit" -j v2.json -t file://$(pwd)/docdoc
```
Create the third version of the JSON document (**v3.json**):
```json
{
	"data" : [1,2,3,4,5,6,7,8,9],
	"value" : "Some other string",
	"version" : "3.0"
}
```
Update again the CRDT:
```bash
./target/debug/libmelda-tools update -a "Amos" -d "Third commit" -j v3.json -t file://$(pwd)/docdoc
```

Finally, create the fourth version of the document (**v4.json**):
```json
{
	"data" : [1,2,3,4,5,6,7,8,9,10],
	"value" : "Yet another string",
	"version" : "4.0"
}
```
Update again the CRDT:
```bash
./target/debug/libmelda-tools update -a "Amos" -d "Fourth commit" -j v4.json -t file://$(pwd)/docdoc
```

You can display the log of the CRDT using the **log** function:
```bash
./target/debug/libmelda-tools log -s file://$(pwd)/docdoc
```
The following will be displayed on the terminal:
```
(A) Block: 62a340029fe5e06a1d13170f7c17801af7c26a87976f53829f6dc747ea2af0d4 (ValidAndApplied)
		Information: {"author":"Amos","description":"Fourth commit"}
		Packs: ["54b7ee82ccd1d82f92f4ef78d4fddd0d719472eb5638889ea88deef2c2fb5f07"]
		Parents: ["4f98690792081209bf3256ad017f189d941653532c16610fdbd4ea720bce1692"]
(-) Block: 4f98690792081209bf3256ad017f189d941653532c16610fdbd4ea720bce1692 (ValidAndApplied)
		Information: {"author":"Amos","description":"Third commit"}
		Packs: ["2a88a9f0b786bf00fec33e3d3bf6c4633d0c8b550187256ba129fdcc29939e8b"]
		Parents: ["cf84352002a3808b8414b95008eb486536e012cc7dfe6a4bfc510a16256715d5"]
(-) Block: cf84352002a3808b8414b95008eb486536e012cc7dfe6a4bfc510a16256715d5 (ValidAndApplied)
		Information: {"author":"Amos","description":"Second commit"}
		Packs: ["2222b429bb89d16c93c4e5bba4a8b082d1ded2df6e2222146bbeaf5b4809f873"]
		Parents: ["707a95e076770ce5b7ddba65d90cf490edde3caf4cc3fc8646b2a0872b5b450a"]
(O) Block: 707a95e076770ce5b7ddba65d90cf490edde3caf4cc3fc8646b2a0872b5b450a (ValidAndApplied)
		Information: {"author":"Amos","description":"First commit"}
		Packs: ["ace7d6a411b127797f2f44a942e095425f1bf5f314c378fcb323f667f22a4a46"]

```

You can also read back the JSON document:
```bash
./target/debug/libmelda-tools read -s file://$(pwd)/docdoc
```
to obtain:
```json
{"_id":"√","data":[1,2,3,4,5,6,7,8,9,10],"value":"Yet another string","version":"4.0"}
```




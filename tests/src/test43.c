// linked list example program
#include <string.h>
#include <sysio.h>

#define NULL 0
int COUNT; // total num nodes added
typedef struct node
{
	int key;
	struct node* next;
} node_t;

void displayAll(node *dlist)
{
    struct node * travel;

    travel = dlist;
    prints("\n------------ Displaying All Nodes----------\n");
    while ( travel != (node*)NULL)
    {
        prints("\nNode with ID: ");
        printi(travel->key);
        travel = travel->next;
    }
}

int main()
{
    node mlist;
    node alist;
    node blist;
    node * travel;
    COUNT = 0;

    mlist.key = 12;
	mlist.next = &alist;
    COUNT = COUNT + 1;

    alist.key = 7;
	alist.next = &blist;
    COUNT = COUNT + 1;

    blist.key = 5;
    blist.next = NULL;
    COUNT = COUNT + 1;

    displayAll(&mlist);


    prints("\nTotal items in the LL: ");
    printi(COUNT);
    prints("\nTest J203 Complete\n");
    return 0;
}

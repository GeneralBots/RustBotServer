let items  = FIND "gb.rob", "ACTION=EMUL1"
FOR EACH item IN items  
    let text = GET "example.com" 
    PRINT item.name
NEXT item 
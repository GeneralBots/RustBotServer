let items  = FIND "gb.rob", "ACTION=EMUL"
FOR EACH item IN items  
    PRINT item.company
    let website = GET WEBSITE item.company  "website"
    PRINT website
    WAIT 10
    let page = GET website

    let prompt = "Create a website for " + item.company + " with the following details: " + page
	
    let alias = REWRITE "Return a single word for {item.company} like a token, no spaces, no special characters, no numbers, no uppercase letters." 

    CREATE SITE item.company + "bot", website, "site", prompt 

    let to = item.emailcto
    let subject = "Simulador criado " + item.company
    let body = "O simulador " + item.company + " foi criado com sucesso. Acesse o site: " + item.company + "bot"

	CREATE DRAFT to, subject, body

NEXT item 